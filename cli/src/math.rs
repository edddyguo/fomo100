use anyhow::{anyhow, Result};
pub const BASE_DECIMAL: u128 = 1_000_000_000; //9

pub mod coin_amount {
    use super::*;

    pub fn raw2display(raw: u128) -> String {
        format!("{}.{:09}", raw / BASE_DECIMAL, raw % BASE_DECIMAL)
    }

    pub fn display2raw(display: &str) -> Result<u128> {
        let split_res: Vec<&str> = display.split('.').collect();
        if split_res.len() == 1 {
            let integer_part = u128::from_str_radix(split_res[0], 10)?;
            Ok(integer_part * BASE_DECIMAL)
        } else if split_res.len() == 2 && split_res[1].len() <= 9 {
            let integer_part = u128::from_str_radix(split_res[0], 10)?;

            let point_part = u128::from_str_radix(split_res[1], 10)?;

            let point_part = point_part * 10u128.pow(9u32 - split_res[1].len() as u32);
            Ok(integer_part * BASE_DECIMAL + point_part)
        } else {
            Err(anyhow!("invailed display number"))?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{coin_amount::*, *};

    #[test]
    fn test_coin_amount() {
        assert_eq!(raw2display(BASE_DECIMAL), "1.000000000".to_string());
        assert_eq!(
            raw2display(BASE_DECIMAL + 100_000_000u128),
            "1.100000000".to_string()
        );
        assert_eq!(
            raw2display(BASE_DECIMAL + 1000u128),
            "1.000001000".to_string()
        );
        assert_eq!(raw2display(1100u128), "0.000001100".to_string());

        assert_eq!(
            display2raw("100.000100000").unwrap(),
            100u128 * BASE_DECIMAL + 100000u128
        );
        assert_eq!(display2raw("1.00").unwrap(), BASE_DECIMAL);
        assert_eq!(display2raw("1").unwrap(), BASE_DECIMAL);
        assert_eq!(
            display2raw("112.000000001").unwrap(),
            112u128 * BASE_DECIMAL + 1
        );
        assert!(display2raw("112.0000000000000000000000000001").is_err());
    }
}
