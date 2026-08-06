use ohlcv_engine::Kline;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct PatternDetection {
    pub pattern_name: String,
    pub pattern_type: String, // BULLISH, BEARISH, NEUTRAL
    pub index: usize,
    pub price_level: f64,
    pub start_time: u64,
    pub end_time: u64,
    pub description: String,
}

pub fn scan_patterns(klines: &[Kline]) -> Vec<PatternDetection> {
    let mut detections = Vec::new();
    let n = klines.len();
    if n < 5 { return detections; }

    for i in 2..n {
        let k1 = &klines[i-2];
        let k2 = &klines[i-1];
        let k3 = &klines[i];

        let body_top = k3.open.max(k3.close);
        let body_bot = k3.open.min(k3.close);
        let body = (body_top - body_bot).max(0.000001);
        let upper_wick = k3.high - body_top;
        let lower_wick = body_bot - k3.low;
        let total_size = (k3.high - k3.low).max(0.000001);

        let k2_body_top = k2.open.max(k2.close);
        let k2_body_bot = k2.open.min(k2.close);
        let k2_body = (k2_body_top - k2_body_bot).max(0.000001);
        let k2_is_green = k2.close > k2.open;
        let is_green = k3.close > k3.open;

        // 1. Pin Bar (Hammer / Shooting Star)
        if lower_wick > body * 2.5 && upper_wick < body * 0.5 {
            detections.push(PatternDetection {
                pattern_name: "Hammer (Pin Bar)".into(),
                pattern_type: "BULLISH".into(),
                index: i, price_level: k3.low,
                start_time: k3.open_time, end_time: k3.close_time,
                description: "Uzun alt iğne, likidite avı (Sweep) veya güçlü alıcı tepkisi.".into()
            });
        } else if upper_wick > body * 2.5 && lower_wick < body * 0.5 {
            detections.push(PatternDetection {
                pattern_name: "Shooting Star (Pin Bar)".into(),
                pattern_type: "BEARISH".into(),
                index: i, price_level: k3.high,
                start_time: k3.open_time, end_time: k3.close_time,
                description: "Uzun üst iğne, likidite avı (Sweep) veya güçlü satıcı baskısı.".into()
            });
        }

        // 2. Engulfing
        if is_green && !k2_is_green && body_bot < k2_body_bot && body_top > k2_body_top {
            detections.push(PatternDetection {
                pattern_name: "Bullish Engulfing".into(),
                pattern_type: "BULLISH".into(),
                index: i, price_level: k3.close,
                start_time: k2.open_time, end_time: k3.close_time,
                description: "Alıcılar önceki kırmızı mumu tamamen yuttu.".into()
            });
        } else if !is_green && k2_is_green && body_top > k2_body_top && body_bot < k2_body_bot {
            detections.push(PatternDetection {
                pattern_name: "Bearish Engulfing".into(),
                pattern_type: "BEARISH".into(),
                index: i, price_level: k3.close,
                start_time: k2.open_time, end_time: k3.close_time,
                description: "Satıcılar önceki yeşil mumu tamamen yuttu.".into()
            });
        }

        // 3. Doji
        if body / total_size < 0.1 && upper_wick > body && lower_wick > body {
            detections.push(PatternDetection {
                pattern_name: "Doji".into(),
                pattern_type: "NEUTRAL".into(),
                index: i, price_level: k3.close,
                start_time: k3.open_time, end_time: k3.close_time,
                description: "Açılış ve kapanış aynı. Piyasada aşırı kararsızlık (Tug-of-war).".into()
            });
        }

        // 4. Inside Bar
        if k3.high < k2.high && k3.low > k2.low {
            detections.push(PatternDetection {
                pattern_name: "Inside Bar".into(),
                pattern_type: "NEUTRAL".into(),
                index: i, price_level: k3.close,
                start_time: k2.open_time, end_time: k3.close_time,
                description: "Fiyat sıkışıyor, kırılım (breakout) hazırlığı.".into()
            });
        }

        // 5. Marubozu
        if body / total_size > 0.95 {
            detections.push(PatternDetection {
                pattern_name: "Marubozu".into(),
                pattern_type: if is_green { "BULLISH".into() } else { "BEARISH".into() },
                index: i, price_level: k3.close,
                start_time: k3.open_time, end_time: k3.close_time,
                description: "İğnesiz dev gövde. Trend yönünde mutlak hakimiyet.".into()
            });
        }

        // 6. Morning / Evening Star
        let k1_is_green = k1.close > k1.open;
        let k1_body_top = k1.open.max(k1.close);
        let k1_body_bot = k1.open.min(k1.close);
        let k1_body = (k1_body_top - k1_body_bot).max(0.000001);
        
        if !k1_is_green && k1_body > total_size * 0.5 && 
           k2_body < k1_body * 0.3 && is_green && k3.close > (k1_body_bot + k1_body_top) / 2.0 {
            detections.push(PatternDetection {
                pattern_name: "Morning Star".into(),
                pattern_type: "BULLISH".into(),
                index: i, price_level: k3.close,
                start_time: k1.open_time, end_time: k3.close_time,
                description: "Düşüş trendi sonunda U-dönüşü.".into()
            });
        } else if k1_is_green && k1_body > total_size * 0.5 && 
                k2_body < k1_body * 0.3 && !is_green && k3.close < (k1_body_bot + k1_body_top) / 2.0 {
             detections.push(PatternDetection {
                 pattern_name: "Evening Star".into(),
                 pattern_type: "BEARISH".into(),
                 index: i, price_level: k3.close,
                 start_time: k1.open_time, end_time: k3.close_time,
                 description: "Yükseliş trendi sonunda U-dönüşü.".into()
             });
        }

        // 7. Tweezer
        let diff_high = (k3.high - k2.high).abs() / k3.high;
        let diff_low = (k3.low - k2.low).abs() / k3.low;
        if diff_high < 0.0001 && upper_wick > body && k2.high - k2_body_top > k2_body {
            detections.push(PatternDetection {
                pattern_name: "Tweezer Tops".into(),
                pattern_type: "BEARISH".into(),
                index: i, price_level: k3.high,
                start_time: k2.open_time, end_time: k3.close_time,
                description: "Aynı fiyattan milimetrik ret yendi. Likidite duvara çarptı.".into()
            });
        } else if diff_low < 0.0001 && lower_wick > body && k2_body_bot - k2.low > k2_body {
            detections.push(PatternDetection {
                pattern_name: "Tweezer Bottoms".into(),
                pattern_type: "BULLISH".into(),
                index: i, price_level: k3.low,
                start_time: k2.open_time, end_time: k3.close_time,
                description: "Aynı fiyattan milimetrik destek bulundu.".into()
            });
        }

        // 10. Dark Cloud Cover / Piercing Line
        if k1_is_green && !is_green && k3.open > k1.high && k3.close < (k1.open + k1.close) / 2.0 {
             detections.push(PatternDetection {
                 pattern_name: "Dark Cloud Cover".into(),
                 pattern_type: "BEARISH".into(),
                 index: i, price_level: k3.close,
                 start_time: k1.open_time, end_time: k3.close_time,
                 description: "Yeşil mumun %50'si aşağı delindi. Güç kaybı.".into()
             });
        } else if !k1_is_green && is_green && k3.open < k1.low && k3.close > (k1.open + k1.close) / 2.0 {
             detections.push(PatternDetection {
                 pattern_name: "Piercing Line".into(),
                 pattern_type: "BULLISH".into(),
                 index: i, price_level: k3.close,
                 start_time: k1.open_time, end_time: k3.close_time,
                 description: "Kırmızı mumun %50'si yukarı delindi. Dönüş sinyali.".into()
             });
        }

        // 11. Spinning Top
        if body / total_size >= 0.1 && body / total_size <= 0.3 && upper_wick > body && lower_wick > body {
            detections.push(PatternDetection {
                 pattern_name: "Spinning Top".into(),
                 pattern_type: "NEUTRAL".into(),
                 index: i, price_level: k3.close,
                 start_time: k3.open_time, end_time: k3.close_time,
                 description: "Alıcı ve satıcı savaşı sürüyor, momentum azaldı.".into()
             });
        }

        // 12. Abandoned Baby
        if k2_body / (k2.high - k2.low).max(0.000001) < 0.1 {
            if k1_is_green && k2.low > k1.high && k3.high < k2.low && !is_green {
                detections.push(PatternDetection {
                     pattern_name: "Bearish Abandoned Baby".into(),
                     pattern_type: "BEARISH".into(),
                     index: i, price_level: k3.close,
                     start_time: k1.open_time, end_time: k3.close_time,
                     description: "Doji mumu boşluklu (Gap) şekilde terk edildi. Çok sert dönüş.".into()
                 });
            } else if !k1_is_green && k2.high < k1.low && k3.low > k2.high && is_green {
                detections.push(PatternDetection {
                     pattern_name: "Bullish Abandoned Baby".into(),
                     pattern_type: "BULLISH".into(),
                     index: i, price_level: k3.close,
                     start_time: k1.open_time, end_time: k3.close_time,
                     description: "Doji mumu boşluklu (Gap) şekilde terk edildi. Çok sert dönüş.".into()
                 });
            }
        }
    }
    
    // 8. 3 White Soldiers / 3 Black Crows
    for i in 2..n {
        let k1 = &klines[i-2];
        let k2 = &klines[i-1];
        let k3 = &klines[i];
        
        let k1_body = (k1.open - k1.close).abs();
        let k2_body = (k2.open - k2.close).abs();
        let k3_body = (k3.open - k3.close).abs();
        
        if k1.close > k1.open && k2.close > k2.open && k3.close > k3.open {
            if k2.close > k1.close && k3.close > k2.close {
                if (k1.high - k1.close) < k1_body * 0.2 && (k2.high - k2.close) < k2_body * 0.2 && (k3.high - k3.close) < k3_body * 0.2 {
                    detections.push(PatternDetection {
                        pattern_name: "3 White Soldiers".into(),
                        pattern_type: "BULLISH".into(),
                        index: i, price_level: k3.close,
                        start_time: k1.open_time, end_time: k3.close_time,
                        description: "Kusursuz ezici alıcı momentumu.".into()
                    });
                }
            }
        }
        
        if k1.close < k1.open && k2.close < k2.open && k3.close < k3.open {
            if k2.close < k1.close && k3.close < k2.close {
                if (k1.close - k1.low) < k1_body * 0.2 && (k2.close - k2.low) < k2_body * 0.2 && (k3.close - k3.low) < k3_body * 0.2 {
                    detections.push(PatternDetection {
                        pattern_name: "3 Black Crows".into(),
                        pattern_type: "BEARISH".into(),
                        index: i, price_level: k3.close,
                        start_time: k1.open_time, end_time: k3.close_time,
                        description: "Kusursuz ezici satıcı momentumu.".into()
                    });
                }
            }
        }
    }

    // 9. Master Candle
    for i in 5..n {
        let master = &klines[i-5];
        let mut is_master = true;
        
        for j in (i-4)..=i {
            if klines[j].high > master.high || klines[j].low < master.low {
                is_master = false;
                break;
            }
        }
        
        if is_master {
             detections.push(PatternDetection {
                 pattern_name: "Master Candle (Akümülasyon)".into(),
                 pattern_type: "NEUTRAL".into(),
                 index: i, price_level: master.close,
                 start_time: master.open_time, end_time: klines[i].close_time,
                 description: format!("Fiyat dev mum içinde sıkıştı. Kırılım yönü sert olacak.")
             });
        }
    }

    detections.sort_by(|a, b| a.index.cmp(&b.index));
    detections
}
