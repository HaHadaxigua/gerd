use std::collections::BTreeMap;

use docker_compose_types;
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use serde_yaml;

type ServiceName = String;

// Module represent an associated services
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Module {
    services: Option<BTreeMap<ServiceName, docker_compose_types::Service>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn parse_service_yaml() {
        let yaml_str = r#"
services:
  prometheus:
    image: '{{ .containers.prometheus.image }}:{{ .containers.prometheus.version }}'
    container_name: prometheus
    restart: 'always'
    network_mode: 'host'
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--storage.tsdb.retention.size={{ self.prometheus_retention_size | int }}GB'
      - '--web.console.libraries=/usr/share/prometheus/console_libraries'
      - '--web.console.templates=/usr/share/prometheus/consoles'
    logging:
      driver: 'json-file'
      options:
        max-size: '10m'
        max-file: '5'
    volumes:
      - '/{{ .service_dir }}/prometheus/data:/prometheus'
      - '/{{ .service_dir }}/prometheus/config:/etc/prometheus'

  blackbox-exporter:
    image: '{{ .containers.blackbox_exporter.image }}:{{ .containers.blackbox_exporter.version }}'
    container_name: blackbox-exporter
    restart: 'always'
    network_mode: 'host'
    command:
      - '--config.file=/blackbox.yml'
    volumes:
      - '/{{ .service_dir }}/blackbox/blackbox.yml:/blackbox.yml:ro'
        "#;

        let module: Module = serde_yaml::from_str(yaml_str).unwrap();
        match module.services {
            Some(s) => {
                println!("got map");
                for (name, service) in &s {
                    println!("{}", &service.image.as_ref().unwrap());
                    println!("{name}")
                }
            }
            _ => println!("not serialize succeed"),
        }
    }

    #[test]
    fn test_quote() {}
}
