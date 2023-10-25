use crate::node::Node;
use anyhow::Result;
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::core::v1::Pod;
use kube::{Client, ResourceExt};
use kube::api::{Api, ListParams};

async fn get_nodes() -> Result<Node> {
    let node: Node = Default::default();

    let k8s_client = Client::try_default().await?;
    let sets: Api<StatefulSet> = Api::default_namespaced(k8s_client.clone());


    match sets.get("tonic-consistent-hashing").await {
        Ok(s) => {println!("name {:?}", s.metadata.name)}
        Err(_) => {}
    };

    //println!(" set {:?}", set.get().unwrap());





    let pods: Api<Pod> = Api::default_namespaced(k8s_client);

    let lp = ListParams::default().labels("helm.sh/chart=grpc-server");
    for p in pods.list(&lp).await? {
        println!("Pod name: {}", p.name_any());
        println!("Pod ip: {:?}", p.status.unwrap().pod_ip.unwrap());
        let cont = p.spec.unwrap().containers;
        for c in cont {
            for p in c.ports.unwrap() {
                println!("Ports {:?} - {:?}", p.name.unwrap(), p.container_port);
            }
        }
    }

    return Ok(node)
}