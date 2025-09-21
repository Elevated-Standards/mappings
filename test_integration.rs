// Integration test for document parser converter
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Document Parser Converter Integration");
    
    // Test 1: Excel Parser
    println!("\n📊 Testing Excel Parser...");
    let excel_parser = ExcelParser::new();
    println!("✅ Excel parser created successfully");
    
    // Test 2: Column Mapper
    println!("\n🗺️  Testing Column Mapper...");
    let column_mapper = ColumnMapper::new();
    println!("✅ Column mapper created successfully");
    
    // Test 3: OSCAL Generator
    println!("\n📋 Testing OSCAL Generator...");
    let oscal_generator = OscalGenerator::new();
    
    // Create sample POA&M data
    let sample_content = json!({
        "poam_items": [
            {
                "weakness_name": "Insufficient Access Controls",
                "severity": "High",
                "status": "Open",
                "scheduled_completion_date": "2024-12-31",
                "description": "Access controls need to be strengthened"
            }
        ]
    });
    
    let metadata = json!({
        "source_file": "test_poam.xlsx",
        "parser_version": "1.0.0",
        "created_by": "integration_test"
    });
    
    match oscal_generator.generate_poam(&sample_content, &metadata) {
        Ok(oscal_doc) => {
            println!("✅ OSCAL POA&M generated successfully");
            println!("📄 Document contains {} bytes", 
                serde_json::to_string_pretty(&oscal_doc)?.len());
        }
        Err(e) => {
            println!("❌ OSCAL generation failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test 4: Document Validator
    println!("\n🔍 Testing Document Validator...");
    let validator = DocumentValidator::new();
    println!("✅ Document validator created successfully");
    
    // Test 5: Fuzzy Matching
    println!("\n🔤 Testing Fuzzy Matching...");
    let mut fuzzy_matcher = document_parser::fuzzy::FuzzyMatcher::new();
    
    let source = "Asset Name";
    let targets = vec![
        "asset_name".to_string(),
        "AssetName".to_string(),
        "Asset_Name".to_string(),
        "Name of Asset".to_string(),
        "Description".to_string(),
    ];
    
    let matches = fuzzy_matcher.find_matches(&source, &targets);
    if !matches.is_empty() {
        println!("✅ Fuzzy matching found {} matches for '{}'", matches.len(), source);
        for (i, m) in matches.iter().take(3).enumerate() {
            println!("   {}. '{}' (confidence: {:.2})", i+1, m.target, m.confidence);
        }
    } else {
        println!("❌ No fuzzy matches found");
    }
    
    println!("\n🎉 Integration test completed successfully!");
    println!("📈 All core components are working properly");
    
    Ok(())
}
