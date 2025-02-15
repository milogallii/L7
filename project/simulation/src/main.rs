use policy_handler::PolicyHandler;
use ship::Ship;
use shipcomponent::ShipComponent;
fn main() {
    // Getting policy parameters for components
    let policy = PolicyHandler::new(String::from("./policies/policy_0.toml"));

    // Setting up ship components accordigly
    let mut ship_components: Vec<ShipComponent> = Vec::new();
    let policy = policy.get_policy();
    policy.iter().for_each(|component| {
        ship_components.push(ShipComponent::new(
            component.name.clone(),
            component.iface.clone(),
            component.mac.clone(),
            component.ip.clone(),
            component.sends.clone(),
            component.receives.clone(),
        ));
    });

    // Setting up ship
    let mut ship = Ship::new(ship_components);
    println!("STARTING SIMULATION");
    ship.monitor_network();

    ship.components.iter().for_each(|component| {
        if component.ifname == "test1" {
            println!(
                "[{}] - [TOTAL SENT: {}Mb] [TOTAL TRANSMITTED: {}Mb]\n{:?}",
                component.name,
                (component.stats.total_sent * 1460.0) / 1000000.0,
                (component.stats.total_received * 1460.0) / 1000000.0,
                &component.stats.bitrate_sent[..5]
            );
        }
    });

    ship.components.iter().for_each(|component| {
        let _ = component
            .stats
            .plot_sent_stats(&format!("{}-{}", component.ifname, component.name));
    });
}
