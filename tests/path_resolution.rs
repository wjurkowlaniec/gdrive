use gdrive::files::path_utils;
use gdrive::hub::Hub;
use std::error::Error;

#[tokio::test]
async fn test_resolve_path() -> Result<(), Box<dyn Error>> {
    let hub = Hub::new()?;
    
    // Test resolving root
    let root = path_utils::resolve_path(&hub, "/").await?;
    assert_eq!(root.id.unwrap(), "root");
    
    // Test resolving a path that should exist
    let result = path_utils::resolve_path(&hub, "/My Drive").await;
    assert!(result.is_ok(), "Should be able to resolve /My Drive");
    
    Ok(())
}

#[tokio::test]
async fn test_resolve_nonexistent_path() {
    let hub = Hub::new().unwrap();
    
    // Test resolving a non-existent path
    let result = path_utils::resolve_path(&hub, "/nonexistent/path").await;
    assert!(matches!(result, Err(path_utils::PathResolutionError::NotFound(_))));
}
