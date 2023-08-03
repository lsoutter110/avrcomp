use std::process::Command;
use std::collections::HashMap;
use std::{fs,path::Path};

pub struct Config {
	pub file_path: String,
    pub comp_proc: String,
    pub upl_proc: String,
    pub freq: String,
    pub prog: String,
    pub linked: Vec<String>,
	pub verbose_output: bool,
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
		let mut verbose_output = false;

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

		if let Some(_opts) = opt_map.get_mut("-v") {
			verbose_output = true;
		}

        Ok(Config { file_path, comp_proc, upl_proc, freq, prog, linked, verbose_output })
    }
}

pub fn run(config: Config) -> Result<(), &'static str> {
	//make bin directory for compilation usage
	if !Path::new("bin").is_dir() {
		match fs::create_dir("bin") {
			Ok(_) => (),
			Err(_) => return Err("Failed to create path 'bin'"),
		}
	}
	//folder prefix
	let f_pre = "bin/";

	println!("Compiling linked files:\n");
	
	//compile linked files
	let mut comp_file_list = Vec::new();
	//push main prog
	comp_file_list.push(format!("{}.c", config.file_path));

	for linked in config.linked {
		let len = linked.len();

		//if filename nonexistant/invalid
		if len<2 {
			return Err("Invalid filename in link arguments!");
		}

		//if not a source file
		if &linked[len-2..] != ".c" {
			comp_file_list.push(String::from(linked));
			continue;
		}

		println!("[avr-gcc] -> {}", linked);
		let mut cmd = Command::new("avr-gcc");
		cmd.arg("-c")
			.arg(format!("-mmcu={}", config.comp_proc))
			.arg(format!("-DF_CPU={}", config.freq))
			// .arg("-Wall")
			.arg("-Os")
			.arg(format!("{}", linked))
			.arg("-o")
			.arg(format!("{}{}.o", f_pre, linked));
		
		if config.verbose_output {
			let arg_vec: Vec<String> = cmd.get_args()
				.map(|x| x.to_str().unwrap().to_string())
				.collect();
			let program = cmd.get_program().to_str().unwrap();
			println!("=> '{} {}'", program, arg_vec.join(" "));
		}

		let output = cmd.output().expect("Failed to execute process");
		
		print!("{}", String::from_utf8_lossy(&output.stderr));
		if !output.status.success() {
			return Err("Failed to compile linked file!");
		}
		println!("Compiled!");
		comp_file_list.push(format!("{}{}.o", f_pre, linked));
	}

	println!("\nCompiling main and uploading:\n");

	println!("[avr-gcc] -> {}.c", config.file_path);
	println!("{:?}", comp_file_list);
	//avr-gcc -mmcu=atmega644p -DF_CPU=12000000 -Wall -Os prog.c -o prog.elf
	let mut cmd = Command::new("avr-gcc");
	cmd.arg(format!("-mmcu={}", config.comp_proc))
		.arg(format!("-DF_CPU={}", config.freq))
		// .arg("-Wall")
		.arg("-Os")
		.args(comp_file_list)
		.arg("-o")
		.arg(format!("{}{}.elf", f_pre, config.file_path));

	if config.verbose_output {
		let arg_vec: Vec<String> = cmd.get_args()
			.map(|x| x.to_str().unwrap().to_string())
			.collect();
		let program = cmd.get_program().to_str().unwrap();
		println!("=> '{} {}'", program, arg_vec.join(" "));
	}
	
	let output = cmd.output().expect("Failed to execute process");
	
	print!("{}", String::from_utf8_lossy(&output.stderr));
	if !output.status.success() {
		return Err("Failed to compile!");
	}
	println!("Compiled!");

	println!("[avr-objcopy] -> {}{}.elf", f_pre, config.file_path);

	//avr-objcopy -O ihex prog.elf prog.hex
	let mut cmd = Command::new("avr-objcopy");
	cmd.arg("-O")
		.arg("ihex")
		.arg(format!("{}{}.elf", f_pre, config.file_path))
		.arg(format!("{}{}.hex", f_pre, config.file_path));

	if config.verbose_output {
		let arg_vec: Vec<String> = cmd.get_args()
			.map(|x| x.to_str().unwrap().to_string())
			.collect();
		let program = cmd.get_program().to_str().unwrap();
		println!("=> '{} {}'", program, arg_vec.join(" "));
	}
			
	let output = cmd.output().expect("Failed to execute process");

	print!("{}", String::from_utf8_lossy(&output.stderr));
	if !output.status.success() {
		return Err("failed to copy objects!");
	}
	println!("Copied!");

	//avrdude -c usbasp -p m644p -U flash:w:prog.hex
	let mut cmd = Command::new("avrdude");
	cmd.arg("-c")
		.arg(config.prog)
		.arg("-p")
		.arg(config.upl_proc)
		.arg("-U")
		.arg(format!("flash:w:{}{}.hex", f_pre, config.file_path));

	if config.verbose_output {
		let arg_vec: Vec<String> = cmd.get_args()
			.map(|x| x.to_str().unwrap().to_string())
			.collect();
		let program = cmd.get_program().to_str().unwrap();
		println!("=> '{} {}'", program, arg_vec.join(" "));
	}
				
	let output = cmd.output().expect("Failed to execute process");

	println!("{}", String::from_utf8_lossy(&output.stderr));
	if !output.status.success() {
		return Err("failed to upload to microcontroller!");
	}

	Ok(())
}