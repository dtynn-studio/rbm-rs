use std::{
    env,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    sync::Arc,
};

use rbm_rs::{
    client::{self, transport, Client},
    module::{chassis, common, robot},
    network::{ConnectionType, NetworkType},
    proto::v1::action::{ActionUpdateHead, V1Action},
    util::host2byte,
};

use tracing::{info, warn};
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
        let detect_client = client::v1::Client::new(tx, vec![rx], vec![rx_closer], host, target)
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

    {
        info!("enable sdk");
        let msg: common::proto::cmd::EnableSdkMode = true.into();
        let resp = device_client.send_cmd(None, msg, true);
        info!("resp: {:?}", resp);
    }

    // get sn
    {
        info!("get sn");
        let msg = common::proto::cmd::GetSN::default();
        let resp = device_client.send_cmd(Some(host2byte(8, 1)), msg, true);
        info!("resp: {:?}", resp);
    }

    // set robot mode
    {
        info!("set robot mode");
        let msg = robot::proto::cmd::Mode::Free;
        let resp = device_client.send_cmd(Some(host2byte(9, 0)), msg, true);
        info!("resp: {:?}", resp);
    }

    // actions
    let actions_dispatcher =
        client::v1::ActionDispatcher::new(device_client).expect("construct actions dispatcher");

    // move action
    {
        info!("start move action");
        let mut move_action =
            chassis::proto::action::Move::<ActionUpdateHead>::new(0.5, 0.0, 0.0, 0.7, 30.0);

        let mut recv_rx = actions_dispatcher
            .send(None, &move_action)
            .expect("send move action cmd");

        while let Some(update) = recv_rx.recv() {
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

    info!("all things done");
}
