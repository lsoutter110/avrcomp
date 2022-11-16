use std::process::Command;
use std::collections::HashMap;

pub struct Config {
	pub file_path: String,
    pub comp_proc: String,
    pub upl_proc: String,
    pub freq: String,
    pub prog: String,
    pub linked: Vec<String>,
}

impl Config {
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("no file path argument"),
        };

        let mut opt_map = HashMap::new();
        let mut curr_opt = String::new();
        let mut curr_vals = Vec::new();
        loop {
        	match args.next() {
        		Some(arg) => {
        			if arg.as_bytes()[0] as char == '-' {
        				opt_map.insert(
        					curr_opt,
        					curr_vals,
        				);
        				curr_opt = arg;
        				curr_vals = Vec::new();

        			} else {
        				curr_vals.push(arg);
        			}
        		},
        		None => {
    				opt_map.insert(
    					curr_opt,
    					curr_vals,
    				);
        			break;
        		}
        	}
        }

        //unwrap opts map

      	let mut comp_proc = String::from("atmega644p");
      	let mut upl_proc = String::from("m644p");
      	let mut freq = String::from("12000000");
      	let mut prog = String::from("usbasp");
        let mut linked = Vec::new();

        if let Some(opts) = opt_map.get_mut("-p") {
        	comp_proc = match opts.pop() {
        		Some(opt) => opt,
        		None => return Err("-p expects two arguments [compilation processor] [upload processor]"),
        	};
        	upl_proc = match opts.pop() {
        		Some(opt) => opt,
        		None => return Err("-p expects two arguments [compilation processor] [upload processor]"),
        	};
        }

        if let Some(opts) = opt_map.get_mut("-c") {
        	prog = match opts.pop() {
        		Some(opt) => opt,
        		None => return Err("-c expects one argument [programmer]"),
        	};
        }

        if let Some(opts) = opt_map.get_mut("-f") {
        	freq = match opts.pop() {
        		Some(opt) => opt,
        		None => return Err("-f expects one argument [frequency]"),
        	};
        }

        if let Some(opts) = opt_map.get_mut("-l") {
        	linked = opts.to_vec();
        }

        Ok(Config { file_path, comp_proc, upl_proc, freq, prog, linked })
    }
}

pub fn run(config: Config) -> Result<(), &'static str> {
	//compile linked files
	let mut comp_file_list = Vec::new();
	for linked in config.linked {
		let output = Command::new("avr-gcc")
									.arg("-c")
									.arg(format!("-mmcu={}", config.comp_proc))
									.arg(format!("-DF_CPU={}", config.freq))
									.arg("-Wall")
									.arg("-Os")
									.arg(format!("{}.c", linked))
									.arg("-o")
									.arg(format!("{}.o", linked))
									.output()
									.expect("Failed to execute process");
		
		if !output.status.success() {
			println!("{}", String::from_utf8_lossy(&output.stderr));
			return Err("failed to compile!");
		}
		println!("[avr-gcc] -> {}.c compiled!", linked);
		comp_file_list.push(format!("{}.o", linked));
	}

	//push main prog
	comp_file_list.push(format!("{}.c", config.file_path));

	//avr-gcc -mmcu=atmega644p -DF_CPU=12000000 -Wall -Os prog.c -o prog.elf
	let output = Command::new("avr-gcc")
								.arg(format!("-mmcu={}", config.comp_proc))
								.arg(format!("-DF_CPU={}", config.freq))
								.arg("-Wall")
								.arg("-Os")
								.args(comp_file_list)
								.arg("-o")
								.arg(format!("{}.elf", config.file_path))
								.output()
								.expect("Failed to execute process");
	
	if !output.status.success() {
		println!("{}", String::from_utf8_lossy(&output.stderr));
		return Err("failed to compile!");
	}
	println!("[avr-gcc] -> {} compiled!", config.file_path);

	//avr-objcopy -O ihex prog.elf prog.hex
	let output = Command::new("avr-objcopy")
								.arg("-O")
								.arg("ihex")
								.arg(format!("{}.elf", config.file_path))
								.arg(format!("{}.hex", config.file_path))
								.output()
								.expect("Failed to execute process");

	if !output.status.success() {
		println!("{}", String::from_utf8_lossy(&output.stderr));
		return Err("failed to copy objects!");
	}
	println!("[avr-objcopy] -> Copied!");

	//avrdude -c usbasp -p m644p -U flash:w:prog.hex
	let output = Command::new("avrdude")
								.arg("-c")
								.arg(config.prog)
								.arg("-p")
								.arg(config.upl_proc)
								.arg("-U")
								.arg(format!("flash:w:{}.hex", config.file_path))
								.output()
								.expect("Failed to execute process");

	println!("{}", String::from_utf8_lossy(&output.stderr));
	if !output.status.success() {
		return Err("failed to upload to microcontroller!");
	}

	Ok(())
}
