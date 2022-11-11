use std::env;
use std::net::{Ipv4Addr, SocketAddr};

use rbm_rs::proto::action::Action;
use tracing::warn;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use rbm_rs::{
    conn::{Client, Udp},
    proto::{host2byte, v1},
};

pub fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();
    let proxy_ip: Ipv4Addr = args[1].parse().expect("parse proxy addr");

    println!("proxy ip: {}", proxy_ip);
    let proxy_addr = format!("{}:30030", proxy_ip)
        .parse()
        .expect("parse proxy addr");
    let device_addr = format!("{}:20020", proxy_ip)
        .parse()
        .expect("parse device addr");
    let bind_addr: SocketAddr = "0.0.0.0:10200".parse().expect("parse bind addr");

    let host = host2byte(9, 6);
    let target = host2byte(9, 0);
    let proxy_client = Client::<v1::V1>::connect::<Udp>(Some(bind_addr), proxy_addr, host, target)
        .expect("connect to proxy port");

    {
        println!("set sdk conn");
        let msg = v1::ctrl::SetSdkConnection {
            host,
            // ip: [192, 168, 2, 21],
            ip: proxy_ip.octets(),
            port: 10200,
            ..Default::default()
        };

        let resp = proxy_client
            .send_cmd(None, msg, None)
            .expect("set sdk conn");
        println!("resp: {:?}", resp);
    }

    drop(proxy_client);

    let device_client =
        Client::<v1::V1>::connect::<Udp>(Some(bind_addr), device_addr, host, target)
            .expect("connect to device port");

    {
        println!("enable sdk");
        let msg: v1::ctrl::SetSdkMode = true.into();
        let resp = device_client.send_cmd(None, msg, None);
        println!("resp: {:?}", resp);
    }

    // get ver
    {
        println!("get product ver");
        let msg = v1::normal::GetProductVersion::default();
        let resp = device_client.send_cmd(Some(host2byte(8, 1)), msg, None);
        println!("resp: {:?}", resp);
    }

    // get sn
    {
        println!("get sn");
        let msg = v1::normal::GetSN::default();
        let resp = device_client.send_cmd(Some(host2byte(8, 1)), msg, None);
        println!("resp: {:?}", resp);
    }

    // move action
    // {
    //     let mut move_action = MoveAction {
    //         x: 0.5,
    //         y: 0.0,
    //         z: 0.0,
    //         spd_xy: 0.7,
    //         spd_z: 30.0,
    //         ..Default::default()
    //     };

    //     let progress_rx = device_client
    //         .send_action(&move_action)
    //         .expect("send action");

    //     while let Some(prog) = progress_rx.next() {
    //         warn!(?prog, "recv progress");
    //         match move_action.apply_progress(prog) {
    //             Ok(completed) => {
    //                 warn!(completed, "move action progressed");
    //                 if completed {
    //                     break;
    //                 }
    //             }

    //             Err(e) => {
    //                 warn!("progress invalid: {:?}", e);
    //                 break;
    //             }
    //         }
    //     }
    // }

    // play sound action
    {
        let mut play_sound =
            v1::action::PlaySoundAction::new(v1::action::RobotSound::SOUND_ID_RECOGNIZED, 3);

        let progress_rx = device_client.send_action(&play_sound).expect("send action");

        while let Some(prog) = progress_rx.next() {
            warn!(?prog, "recv progress");
            match play_sound.apply_progress(prog) {
                Ok(completed) => {
                    warn!(completed, "sound action progressed");
                    if completed {
                        break;
                    }
                }

                Err(e) => {
                    warn!("progress invalid: {:?}", e);
                    break;
                }
            }
        }
    }
}
