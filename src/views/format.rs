/// Turns our stored YYYY-MM-DD into the British DD-MM-YYYY format for
/// display only, storage and date inputs stay in ISO format throughout.
pub fn to_british_date(iso: &str) -> String {
    let parts: Vec<&str> = iso.split('-').collect();
    if parts.len() == 3 {
        format!("{}-{}-{}", parts[2], parts[1], parts[0])
    } else {
        iso.to_string()
    }
}