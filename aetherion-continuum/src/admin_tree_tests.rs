//! Unit tests for the Administrative Field System

use aetherion_continuum::{AdminTree, AdministrativeNode, AdminLevel, Restriction};
use geo::{MultiPolygon, Polygon, Coord};

#[test]
fn test_admin_hierarchy_creation() {
    let mut tree = AdminTree::default();
    
    // Create county
    let county_coords = vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 10.0, y: 0.0 },
        Coord { x: 10.0, y: 10.0 },
        Coord { x: 0.0, y: 10.0 },
        Coord { x: 0.0, y: 0.0 },
    ];
    let county_poly = MultiPolygon::new(vec![Polygon::new(county_coords, vec![])]);
    let county = AdministrativeNode::new(
        "county_001".to_string(),
        "Test County".to_string(),
        AdminLevel::County,
        county_poly,
    );
    tree.add_node(county);
    
    // Create city within county
    let city_coords = vec![
        Coord { x: 2.0, y: 2.0 },
        Coord { x: 8.0, y: 2.0 },
        Coord { x: 8.0, y: 8.0 },
        Coord { x: 2.0, y: 8.0 },
        Coord { x: 2.0, y: 2.0 },
    ];
    let city_poly = MultiPolygon::new(vec![Polygon::new(city_coords, vec![])]);
    let mut city = AdministrativeNode::new(
        "city_001".to_string(),
        "Test City".to_string(),
        AdminLevel::City,
        city_poly,
    );
    city.parent_id = Some("county_001".to_string());
    tree.add_node(city);
    
    // Verify hierarchy
    assert_eq!(tree.nodes.len(), 2);
    let city_node = tree.nodes.get("city_001").unwrap();
    assert_eq!(city_node.parent_id, Some("county_001".to_string()));
}

#[test]
fn test_rule_priority_override() {
    let mut tree = AdminTree::default();
    
    // County sets base speed limit (priority 10)
    let county_coords = vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 10.0, y: 0.0 },
        Coord { x: 10.0, y: 10.0 },
        Coord { x: 0.0, y: 10.0 },
        Coord { x: 0.0, y: 0.0 },
    ];
    let county_poly = MultiPolygon::new(vec![Polygon::new(county_coords, vec![])]);
    let mut county = AdministrativeNode::new(
        "county_001".to_string(),
        "Test County".to_string(),
        AdminLevel::County,
        county_poly,
    );
    
    // Manually add a rule to test override
    county.rule_stack.apply(Restriction {
        key: "max_speed".to_string(),
        value: "55_mph".to_string(),
        priority: 10,
        time_of_day: None,
    });
    tree.add_node(county);
    
    // Neighborhood sets stricter speed limit (priority 40)
    let neighborhood_coords = vec![
        Coord { x: 3.0, y: 3.0 },
        Coord { x: 7.0, y: 3.0 },
        Coord { x: 7.0, y: 7.0 },
        Coord { x: 3.0, y: 7.0 },
        Coord { x: 3.0, y: 3.0 },
    ];
    let neighborhood_poly = MultiPolygon::new(vec![Polygon::new(neighborhood_coords, vec![])]);
    let mut neighborhood = AdministrativeNode::new(
        "neighborhood_001".to_string(),
        "Quiet Heights".to_string(),
        AdminLevel::Neighborhood,
        neighborhood_poly,
    );
    neighborhood.parent_id = Some("county_001".to_string());
    
    // Override with lower speed limit
    neighborhood.rule_stack.apply(Restriction {
        key: "max_speed".to_string(),
        value: "25_mph".to_string(),
        priority: 40,
        time_of_day: None,
    });
    tree.add_node(neighborhood);
    
    // Get effective rules for neighborhood
    let rules = tree.get_effective_rules("neighborhood_001");
    
    // Neighborhood rule should override county rule due to higher priority
    assert!(rules.restrictions.contains_key("max_speed"));
    let speed_rule = rules.restrictions.get("max_speed").unwrap();
    assert_eq!(speed_rule.value, "25_mph");
    assert_eq!(speed_rule.priority, 40);
}

#[test]
fn test_point_in_boundary_query() {
    let mut tree = AdminTree::default();
    
    // Create nested boundaries: County > City > Neighborhood
    let county_coords = vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 100.0, y: 0.0 },
        Coord { x: 100.0, y: 100.0 },
        Coord { x: 0.0, y: 100.0 },
        Coord { x: 0.0, y: 0.0 },
    ];
    let county_poly = MultiPolygon::new(vec![Polygon::new(county_coords, vec![])]);
    let county = AdministrativeNode::new(
        "county_001".to_string(),
        "Large County".to_string(),
        AdminLevel::County,
        county_poly,
    );
    tree.add_node(county);
    
    let city_coords = vec![
        Coord { x: 20.0, y: 20.0 },
        Coord { x: 80.0, y: 20.0 },
        Coord { x: 80.0, y: 80.0 },
        Coord { x: 20.0, y: 80.0 },
        Coord { x: 20.0, y: 20.0 },
    ];
    let city_poly = MultiPolygon::new(vec![Polygon::new(city_coords, vec![])]);
    let mut city = AdministrativeNode::new(
        "city_001".to_string(),
        "Central City".to_string(),
        AdminLevel::City,
        city_poly,
    );
    city.parent_id = Some("county_001".to_string());
    tree.add_node(city);
    
    // Query point inside city
    let point_inside_city = geo::Point::new(50.0, 50.0);
    let deepest = tree.find_deepest_node(&point_inside_city).unwrap();
    assert_eq!(deepest.level, AdminLevel::City);
    assert_eq!(deepest.name, "Central City");
    
    // Query point only in county (outside city)
    let point_in_county_only = geo::Point::new(5.0, 5.0);
    let deepest = tree.find_deepest_node(&point_in_county_only).unwrap();
    assert_eq!(deepest.level, AdminLevel::County);
    assert_eq!(deepest.name, "Large County");
}

#[test]
fn test_resource_aggregation() {
    let mut tree = AdminTree::default();
    
    // Create parent node
    let county_coords = vec![
        Coord { x: 0.0, y: 0.0 },
        Coord { x: 10.0, y: 0.0 },
        Coord { x: 10.0, y: 10.0 },
        Coord { x: 0.0, y: 10.0 },
        Coord { x: 0.0, y: 0.0 },
    ];
    let county_poly = MultiPolygon::new(vec![Polygon::new(county_coords, vec![])]);
    let mut county = AdministrativeNode::new(
        "county_001".to_string(),
        "Test County".to_string(),
        AdminLevel::County,
        county_poly,
    );
    
    // Create child nodes with resources
    let city1_coords = vec![
        Coord { x: 1.0, y: 1.0 },
        Coord { x: 4.0, y: 1.0 },
        Coord { x: 4.0, y: 4.0 },
        Coord { x: 1.0, y: 4.0 },
        Coord { x: 1.0, y: 1.0 },
    ];
    let city1_poly = MultiPolygon::new(vec![Polygon::new(city1_coords, vec![])]);
    let mut city1 = AdministrativeNode::new(
        "city_001".to_string(),
        "City One".to_string(),
        AdminLevel::City,
        city1_poly,
    );
    city1.resource_pool.population = 10000;
    city1.resource_pool.tax_revenue = 50000.0;
    city1.parent_id = Some("county_001".to_string());
    
    let city2_coords = vec![
        Coord { x: 6.0, y: 6.0 },
        Coord { x: 9.0, y: 6.0 },
        Coord { x: 9.0, y: 9.0 },
        Coord { x: 6.0, y: 9.0 },
        Coord { x: 6.0, y: 6.0 },
    ];
    let city2_poly = MultiPolygon::new(vec![Polygon::new(city2_coords, vec![])]);
    let mut city2 = AdministrativeNode::new(
        "city_002".to_string(),
        "City Two".to_string(),
        AdminLevel::City,
        city2_poly,
    );
    city2.resource_pool.population = 15000;
    city2.resource_pool.tax_revenue = 75000.0;
    city2.parent_id = Some("county_001".to_string());
    
    tree.add_node(county);
    tree.add_node(city1);
    tree.add_node(city2);
    
    // Aggregate resources manually for testing
    let children: Vec<&AdministrativeNode> = vec![&city1, &city2];
    county.aggregate_resources(&children);
    
    // Verify aggregation
    assert_eq!(county.resource_pool.population, 25000);
    assert_eq!(county.resource_pool.tax_revenue, 125000.0);
}
