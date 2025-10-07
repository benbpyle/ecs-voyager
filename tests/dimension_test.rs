#[cfg(test)]
mod tests {
    use aws_sdk_cloudwatch::types::Dimension;

    /// Helper function to create a CloudWatch Dimension (mimics the one in aws.rs)
    fn create_dimension(name: &str, value: &str) -> Dimension {
        Dimension::builder().name(name).value(value).build()
    }

    #[test]
    fn test_dimension_builder_with_both_fields() {
        // Test with both name and value - this is the correct usage
        let dim = create_dimension("ServiceName", "my-service");

        // Verify both fields are set
        assert_eq!(dim.name(), Some("ServiceName"));
        assert_eq!(dim.value(), Some("my-service"));
    }

    #[test]
    fn test_dimension_builder_cluster_name() {
        // Test creating a ClusterName dimension
        let dim = create_dimension("ClusterName", "my-cluster");

        assert_eq!(dim.name(), Some("ClusterName"));
        assert_eq!(dim.value(), Some("my-cluster"));
    }

    #[test]
    fn test_dimension_with_special_characters() {
        // Test dimension values with hyphens, underscores, and other valid characters
        let dim = create_dimension("ServiceName", "my-service_v1.0");

        assert_eq!(dim.name(), Some("ServiceName"));
        assert_eq!(dim.value(), Some("my-service_v1.0"));
    }

    #[test]
    fn test_dimension_with_empty_strings() {
        // Test that empty strings are still set (even though AWS will reject them)
        let dim = create_dimension("", "");

        // The builder accepts empty strings, but AWS API will reject them
        assert_eq!(dim.name(), Some(""));
        assert_eq!(dim.value(), Some(""));
    }

    #[test]
    fn test_multiple_dimensions() {
        // Test creating multiple dimensions as used in get_service_metrics
        let service_dim = create_dimension("ServiceName", "my-service");
        let cluster_dim = create_dimension("ClusterName", "my-cluster");

        // Verify both dimensions are independent
        assert_eq!(service_dim.name(), Some("ServiceName"));
        assert_eq!(service_dim.value(), Some("my-service"));
        assert_eq!(cluster_dim.name(), Some("ClusterName"));
        assert_eq!(cluster_dim.value(), Some("my-cluster"));
    }

    #[test]
    fn test_dimension_clone() {
        // Test that dimensions can be cloned (needed for reuse in multiple metrics)
        let original = create_dimension("ServiceName", "my-service");
        let cloned = original.clone();

        assert_eq!(original.name(), cloned.name());
        assert_eq!(original.value(), cloned.value());
    }

    #[test]
    fn test_dimension_builder_optional_fields() {
        // This test demonstrates that the builder allows optional fields
        // but this creates invalid dimensions that AWS will reject
        let dim_no_value = Dimension::builder().name("TestName").build();

        let dim_no_name = Dimension::builder().value("TestValue").build();

        let dim_empty = Dimension::builder().build();

        // All of these compile but create invalid dimensions
        assert_eq!(dim_no_value.name(), Some("TestName"));
        assert_eq!(dim_no_value.value(), None); // Invalid for AWS API

        assert_eq!(dim_no_name.name(), None); // Invalid for AWS API
        assert_eq!(dim_no_name.value(), Some("TestValue"));

        assert_eq!(dim_empty.name(), None); // Invalid for AWS API
        assert_eq!(dim_empty.value(), None); // Invalid for AWS API
    }

    #[test]
    fn test_dimension_with_long_values() {
        // Test dimension with maximum allowed lengths
        // Name: 1-255 characters, Value: 1-1024 characters
        let long_name = "A".repeat(255);
        let long_value = "B".repeat(1024);

        let dim = create_dimension(&long_name, &long_value);

        assert_eq!(dim.name(), Some(long_name.as_str()));
        assert_eq!(dim.value(), Some(long_value.as_str()));
    }

    #[test]
    fn test_ecs_specific_dimensions() {
        // Test the specific dimension names used by ECS metrics
        let service_dim = create_dimension("ServiceName", "example-service");
        let cluster_dim = create_dimension("ClusterName", "example-cluster");

        // These are the correct dimension names for ECS service metrics
        assert_eq!(service_dim.name(), Some("ServiceName"));
        assert_eq!(cluster_dim.name(), Some("ClusterName"));
    }
}
