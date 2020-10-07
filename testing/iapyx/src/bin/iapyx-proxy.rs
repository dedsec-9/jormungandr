use iapyx::cli::args::proxy::IapyxProxyCommand;
use tokio;
use structopt::StructOpt;
use warp::Filter;
use std::net::SocketAddr;
use warp_reverse_proxy::reverse_proxy_filter;

#[tokio::main]
async fn main() {
    let server_stub =  IapyxProxyCommand::from_args().build().unwrap();

    let root = warp::path!("api"/ "v0" / ..);

   
       let proposals = warp::path!("proposals").and(
            reverse_proxy_filter("".to_string(), server_stub.http_vit_address().to_string()),
        );

        let fund = warp::path!("fund" / ..).and(
            reverse_proxy_filter("".to_string(), server_stub.http_vit_address().to_string()),
        );

        let account = warp::path!("account" / ..).and(
            reverse_proxy_filter("".to_string(), server_stub.http_node_address().to_string()),
        );

        let fragment = warp::path!("fragments" / ..).and(
            reverse_proxy_filter("".to_string(), server_stub.http_node_address().to_string()),
        );

        
        let message = warp::path!("message" / ..).and(
            reverse_proxy_filter("".to_string(), server_stub.http_node_address().to_string()),
        );

        let settings = warp::path!("settings" / ..).and(
            reverse_proxy_filter("".to_string(), server_stub.http_node_address().to_string()),
        );

        let explorer = warp::path!("explorer" / "graphql").and(
            reverse_proxy_filter("".to_string(), server_stub.http_node_address().to_string()),
        );

        let block0_content = server_stub.block0().clone();    
            
        let block0 = warp::path!("block0").map(move|| Ok(block0_content.clone()) );

        let app = root.and(
            proposals
                .or(fund)
                .or(account)
                .or(fragment)
                .or(message)
                .or(settings)
                .or(explorer)
            )
        .or(block0);

        warp::serve(app).run(server_stub.base_address()).await;
}
