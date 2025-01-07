use policy_parser::PolicyParser;
use ship::Ship;
use shipcomponent::ShipComponent;

fn main() {
    // Getting policy parameters for components
    let policy_parser = PolicyParser::new(String::from("./policies/policy_0.toml"));
    policy_parser.show_policy();

    // Setting up ship components accordigly
    let mut ship_components: Vec<ShipComponent> = Vec::new();
    let policy = policy_parser.get_policy();
    policy.iter().for_each(|component| {
        ship_components.push(ShipComponent::new(
            component.name.clone(),
            component.iface.clone(),
            component.sends.clone(),
            component.receives.clone(),
        ));
    });

    // // Setting up ship
    let mut ship = Ship::new(ship_components);
    ship.monitor_network();
}
