use crate::node::Node;
use anyhow::{Result, Error};
use k8s_openapi::api::apps::v1::StatefulSet;
use k8s_openapi::api::core::v1::Pod;
use kube::{Client, ResourceExt};
use kube::api::{Api, ListParams};

pub async fn get_nodes(pod_label: &str, statefulset_name: &str, port_name: &str) -> Result<Vec<Node>> {
    let mut nodes: Vec<Node> = Vec::new();

    let k8s_client = Client::try_default().await?;
    let sets: Api<StatefulSet> = Api::default_namespaced(k8s_client.clone());
    let s = sets.get(statefulset_name).await?;

    println!("name {:?}", s.metadata.name);
    let name = s.metadata.name.ok_or(Error::msg("statefulset"))?;
    if name == statefulset_name {
        let pods: Api<Pod> = Api::default_namespaced(k8s_client);

        let lp = ListParams::default().labels(pod_label);
        for p in pods.list(&lp).await? {
            let ip = p.clone().status.ok_or(Error::msg("pod status"))?.pod_ip.ok_or(Error::msg("pod ip"))?;
            println!("Pod name: {}", p.clone().name_any());
            println!("Pod ip: {:?}", p.clone().status.unwrap().pod_ip.unwrap());
            let cont = p.spec.ok_or(Error::msg("spec containers"))?.containers;
            for c in cont {
                for p in c.ports.ok_or(Error::msg("port"))? {
                    let p_name = p.clone().name.ok_or(Error::msg("port name"))?;
                    if p_name == port_name {
                        println!("Ports {:?} - {:?}", p.clone().name.unwrap(), p.container_port);
                        nodes.push(Node::new(ip.as_str(), p.container_port as u16));
                    } else {
                        return Err(Error::msg(format!("port: {} not found", port_name)));
                    }
                }
            }
        }
    }
    return Ok(nodes);
}