use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use rust_decimal::prelude::*;
use crate::ring_buffer::{OwnedEvent, EventType};
use crate::memory::order_ring::OrderRingBuffer;
use std::sync::Arc;

pub struct PythonStrategyEngine {
    py_module: Py<PyModule>,
    order_ring: Arc<OrderRingBuffer>,
}

impl PythonStrategyEngine {
    pub fn new(script_path: &str, order_ring: Arc<OrderRingBuffer>) -> PyResult<Self> {
        let code = std::fs::read_to_string(script_path)?;
        
        Python::with_gil(|py| {
            let module = PyModule::from_code_bound(py, &code, "strategy.py", "strategy")?;
            
            if module.getattr("on_start").is_ok() {
                module.call_method0("on_start")?;
            }
            
            Ok(Self {
                py_module: module.unbind(),
                order_ring,
            })
        })
    }

    pub fn on_event(&self, event: &OwnedEvent) -> PyResult<()> {
        Python::with_gil(|py| {
            let py_module = self.py_module.bind(py);
            
            let dict = PyDict::new_bound(py);
            // Decode symbol
            let symbol_str = std::str::from_utf8(&event.symbol).unwrap_or("UNKNOWN").trim_end_matches('\0');
            dict.set_item("symbol", symbol_str)?;
            
            match event.payload {
                EventType::Trade { price, quantity, timestamp, is_buyer_maker } => {
                    dict.set_item("type", "trade")?;
                    dict.set_item("price", price.to_f64().unwrap_or(0.0))?;
                    dict.set_item("quantity", quantity.to_f64().unwrap_or(0.0))?;
                    dict.set_item("timestamp", timestamp)?;
                    dict.set_item("is_buyer_maker", is_buyer_maker)?;
                },
                EventType::BookTicker { best_bid_price, best_ask_price, .. } => {
                    dict.set_item("type", "book_ticker")?;
                    dict.set_item("bid_price", best_bid_price.to_f64().unwrap_or(0.0))?;
                    dict.set_item("ask_price", best_ask_price.to_f64().unwrap_or(0.0))?;
                },
                _ => return Ok(()),
            }

            if let Ok(func) = py_module.getattr("on_tick") {
                let args = (dict,);
                if let Err(e) = func.call1(args) {
                    eprintln!("Python on_tick hatası: {:?}", e);
                }
            }
            Ok(())
        })
    }
}
