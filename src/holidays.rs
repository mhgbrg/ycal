use clap::Parser;

#[derive(Parser)]
#[command(about = "Fetch public holidays from Nager.Date and output as special days JSON")]
struct Cli {
    /// Year to fetch holidays for
    year: i32,
    /// Country code (e.g. GB, SE, DE)
    country_code: String,
}

fn main() {
    let cli = Cli::parse();

    let special_days = ycal::fetch_holidays(cli.year, &cli.country_code).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let json = serde_json::to_string_pretty(&special_days).unwrap();
    println!("{}", json);
}
