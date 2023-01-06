use std::{
    env,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    sync::Arc,
};

use rbm_rs::{
    client::{self, transport, Connection},
    module::{chassis, common},
    network::{ConnectionType, NetworkType},
    product::robot,
    proto::{ProtoAction, ProtoSubscribe},
    util::host2byte,
};

use crossbeam_channel::{bounded, select};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = env::args().collect();
    let proxy_ip: Ipv4Addr = args[1].parse().expect("parse proxy addr");

    info!("proxy ip: {}", proxy_ip);
    let detect_addr: SocketAddr = format!("{}:30030", proxy_ip)
        .parse()
        .expect("parse detect addr");
    let device_addr: SocketAddr = format!("{}:20020", proxy_ip)
        .parse()
        .expect("parse device addr");
    let bind_port = 10200;
    let bind_addr: SocketAddr = format!("0.0.0.0:{}", bind_port)
        .parse()
        .expect("parse bind addr");
    let proxy_addr: SocketAddrV4 = format!("{}:{}", proxy_ip, bind_port)
        .parse()
        .expect("parse proxy addr");

    let host = host2byte(9, 6);
    let target = host2byte(9, 0);

    let socket = UdpSocket::bind(bind_addr)
        .map(Arc::new)
        .expect("udp socket bind");

    // detect ip and set connection
    {
        let (rx, rx_closer) =
            transport::udp::trans_rx(socket.clone()).expect("construct udp trans rx");
        let tx = transport::udp::trans_tx_to(socket.clone(), detect_addr);
        let detect_client =
            client::v1::Connection::new(tx, vec![rx], vec![rx_closer], host, target)
                .expect("construct v1 detect client");

        let detect_req = common::proto::cmd::SetSdkConnection {
            host,
            network: NetworkType::Sta,
            connection: ConnectionType::Udp,
            addr: proxy_addr,
        };

        let detect_resp = detect_client
            .send_cmd(None, detect_req, true)
            .expect("send detect req");
        info!("detected ip: {:?}", detect_resp);

        drop(detect_client);
    }

    let (rx, rx_closer) = transport::udp::trans_rx(socket.clone()).expect("construct udp trans rx");
    let tx = transport::udp::trans_tx_to(socket, device_addr);

    let device_client = client::v1::Client::new(tx, vec![rx], vec![rx_closer], host, target)
        .map(Arc::new)
        .expect("construct v1 device client");

    let mut ep =
        robot::RobotMasterEP::new(device_client).expect("construct RobotMasterEP instance");

    let version = ep.common.version().expect("get product version");
    info!("product version: {:?}", version);

    let sn = ep.common.sn().expect("get serial number");
    info!("product serial number: {}", sn);

    let (mut pos, mut pos_rx, subscription) = ep
        .chassis
        .subscribe_position(chassis::proto::subscribe::PositionOriginMode::Current, None)
        .expect("subscribe chassis position");

    let (all_done_tx, all_done_rx) = bounded::<()>(0);

    let join = std::thread::spawn(move || {
        // chassis position
        {
            let mut count = 0;
            loop {
                select! {
                    recv(pos_rx.inner()) -> incoming_rx => {
                        info!("recv incoming position update");
                        let incoming = match incoming_rx {
                            Ok(i) => i,
                            Err(e) => {
                                error!("error incoming {:?}", e);
                                continue;
                            }
                        };

                        count += 1;

                        pos.apply_push(incoming).unwrap_or_else(|e| {
                            panic!(
                                "applying incoming update #{} {:?}: {:?}",
                                count, incoming, e
                            )
                        });

                        info!("applied position: {:?}", pos.current);

                        if count == 10 {
                            break;
                        }
                    }

                    recv(all_done_rx) -> _ => {
                        break;
                    }
                }
            }
            // while let Some(incoming) = subscription.rx.recv() {
            // }

            drop(subscription);
        }
    });

    // actions
    // move action
    {
        info!("start move action");
        let (mut move_action, mut move_update_rx) = ep
            .chassis
            .action_start_move(0.5, 0.0, 0.0, Some(0.7), None)
            .expect("start move action");

        while let Some(update) = move_update_rx.recv() {
            match move_action.apply_update(update) {
                Ok(done) => {
                    info!("move progress: {:?}", move_action.progress);
                    if done {
                        break;
                    }
                }

                Err(e) => {
                    warn!("apply move update: {:?}", e);
                }
            }
        }
    }

    std::thread::sleep(std::time::Duration::from_secs(5));
    drop(all_done_tx);
    join.join().expect("position sub thread join");
    info!("all things done");
}
