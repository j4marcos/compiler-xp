use std::env;


pub  fn zeller() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!("Uso: {} DD MM CC YY", args[0]);
        return;
    }
    let dd: i32 = args[1].parse().expect("DD inválido");
    let mm: i32 = args[2].parse().expect("MM inválido");
    let cc: i32 = args[3].parse().expect("CC inválido");
    let yy: i32 = args[4].parse().expect("YY inválido");

    let week_day = (dd + ((13 * (mm + 1)) / 5) + yy + (yy / 4) + (cc / 4) - 2 * cc) % 7;

    println!("{}", week_day);
}