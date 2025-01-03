mod l7_components;
use crate::l7_components::ship::Ship;
use crate::l7_components::ship_components::ShipComponent;

fn main() {
    // Setting up the ship components
    let c0 = ShipComponent::new(String::from("girobussola"), String::from("test1"));
    let c1 = ShipComponent::new(String::from("ais"), String::from("test2"));
    let c2 = ShipComponent::new(String::from("gps"), String::from("test3"));
    let c3 = ShipComponent::new(String::from("ecoscandaglio"), String::from("test4"));
    let c4 = ShipComponent::new(String::from("velocita"), String::from("test5"));
    let c5 = ShipComponent::new(String::from("radar"), String::from("test6"));
    let c6 = ShipComponent::new(String::from("ecdis"), String::from("test7"));

    // Setting up ship
    let components: Vec<ShipComponent> = vec![c0, c1, c2, c3, c4, c5, c6];
    let mut ship = Ship::new(components);
    ship.monitor_components();
}
