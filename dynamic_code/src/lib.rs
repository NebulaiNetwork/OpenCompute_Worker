use rune::alloc::String as RuneString;
use rune::runtime::{Args, FromValue, GuardedArgs, Shared, Vm, VmResult};
// use rune::termcolor::{ColorChoice, StandardStream, Buffer};
use rune::termcolor::Buffer;
use rune::{Module, Diagnostics, Source, Sources, Value, ContextError};
use rune_modules::default_context;

use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

pub use rune;

pub static MODULE_REGISTRY: Lazy<Mutex<Vec<Box<dyn Fn(&mut Module) -> Result<(), ContextError> + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn register_rust_function_i64(name: &'static str, func: fn(i64) -> i64) {
    let register = Box::new(move |module: &mut Module| {
        module.function([name], func).build()?;
        Ok(())
    });
    MODULE_REGISTRY.lock().unwrap().push(register);
}

pub fn register_rust_function_matrix(name: &'static str, func: fn(Vec<Vec<f32>>, Vec<Vec<f32>>) -> Vec<Vec<f32>>) {
    let register = Box::new(move |module: &mut Module| {
        module.function([name], func).build()?;
        Ok(())
    });
    MODULE_REGISTRY.lock().unwrap().push(register);
}

pub struct DynamicCode {
    vm: Vm,
    have_init: bool,
}

impl DynamicCode {
    pub fn new(script: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut context = default_context()?;

        let mut module = Module::new();
        for reg in MODULE_REGISTRY.lock().unwrap().iter() {
            reg(&mut module)?;
        }
        
        context.install(module)?;

        let mut sources = Sources::new();
        let _ = sources.insert(Source::new("main", script).expect("invalid source"));

        let mut diagnostics = Diagnostics::new();

        let compilation = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        let unit = match compilation {
            Ok(unit) => unit,
            Err(_) => {
                // let mut writer = StandardStream::stderr(ColorChoice::Always);
                // diagnostics.emit(&mut writer, &sources)?;

                let mut buffer = Buffer::no_color();
                diagnostics.emit(&mut buffer, &sources)?;

                let output = String::from_utf8_lossy(buffer.as_slice()).to_string();
                return Err(output.into());
            }
        };

        let runtime_context = Arc::new(context.runtime()?);
        let vm = Vm::new(runtime_context, Arc::new(unit));

        Ok(DynamicCode {
            vm: vm,
            have_init: true,
        })
    }

    pub fn use_func<T, A>(
        &mut self,
        func_name: &str,
        args: A,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: FromValue,
        A: Args + GuardedArgs,
    {
        if !self.have_init {
            return Err("dynamic code have not init".into());
        }

        let output = self.vm.call([func_name], args)?;

        match T::from_value(output) {
            VmResult::Ok(v) => Ok(v),
            VmResult::Err(e) => Err(e.into()),
        }
    }

    pub async fn use_func_dyn<T: FromValue>(&mut self, func: &str, args_json: &str) -> Result<T, Box<dyn std::error::Error>> {
        let json: serde_json::Value = serde_json::from_str(args_json)?;
    
        let args: Vec<Value> = match &json {
            serde_json::Value::Array(arr) => {
                arr.iter().map(json_to_rune).collect::<Result<_, _>>()?
            }
            _ => vec![json_to_rune(&json)?],
        };

        let output = match args.len() {
            // 0 => self.vm.call([func], ())?,
            // 1 => self.vm.call([func], (args[0].clone(),))?,
            // 2 => self.vm.call([func], (args[0].clone(), args[1].clone()))?,
            // 3 => self.vm.call([func], (args[0].clone(), args[1].clone(), args[2].clone()))?,
            // 4 => self.vm.call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone()))?,
            // 5 => self.vm.call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone()))?,
            // 6 => self.vm.call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone()))?,
            // 7 => self.vm.call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone()))?,
            // 8 => self.vm.call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone(), args[6].clone()))?,
            0 => self.vm.async_call([func], ()).await?,
            1 => self.vm.async_call([func], (args[0].clone(),)).await?,
            2 => self.vm.async_call([func], (args[0].clone(), args[1].clone())).await?,
            3 => self.vm.async_call([func], (args[0].clone(), args[1].clone(), args[2].clone())).await?,
            4 => self.vm.async_call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone())).await?,
            5 => self.vm.async_call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone())).await?,
            6 => self.vm.async_call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone())).await?,
            7 => self.vm.async_call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone(), args[6].clone())).await?,
            8 => self.vm.async_call([func], (args[0].clone(), args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone(), args[5].clone(), args[6].clone(), args[7].clone())).await?,
            _ => return Err("Too many arguments".into()),
        };
    
        match T::from_value(output) {
            VmResult::Ok(v) => Ok(v),
            VmResult::Err(e) => Err(e.into()),
        }
    }

}


fn json_to_rune(value: &serde_json::Value) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(match value {
        serde_json::Value::Null => Value::from(()),
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(f) = n.as_f64() {
                Value::from(f)
            } else {
                return Err("Unsupported number format".into());
            }
        }
        serde_json::Value::String(s) => {
            Value::String(Shared::new(RuneString::try_from(s.as_str())?)?)
        }

        serde_json::Value::Array(arr) => {
            let mut vec = rune::runtime::Vec::with_capacity(arr.len())?;
            for v in arr {
                vec.push(json_to_rune(v)?)?;
            }
            let shared_vec = Shared::new(vec)?;
            Value::from(shared_vec)
        }
        serde_json::Value::Object(_obj) => return Err("Objects are not supported".into()),
        // _ => unreachable!("Unexpected Value variant"),
    })
}