use substreams::log;

pub fn append_0x(i: &str) -> String {
    format!("0x{}", i)
}

pub fn compare_tokens(token_x: &str, token_y: &str) -> bool {
    let value: bool = match token_x.cmp(&token_y) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Equal => {
            log::info!("TokenX : {}", &token_x.to_string());
            log::info!("TokenY : {}", &token_y.to_string());
            panic!("Shouldn't be equal")
        }
        std::cmp::Ordering::Greater => false,
    };
    value
}

pub fn get_sorted_token0(token_x: &str, token_y: &str) -> String {
    let value: bool = compare_tokens(token_x, token_y);
    if value == false {
        return token_x.to_string();
    } else {
        token_y.to_string()
    }
}

pub fn get_sorted_token1(token_x: &str, token_y: &str) -> String {
    let value: bool = compare_tokens(token_x, token_y);
    if value == false {
        return token_y.to_string();
    } else {
        token_x.to_string()
    }
}

pub fn get_sorted_price(token_x: &str, token_y: &str, value_0: &str, value_1: &str) -> String {
    let value: bool = compare_tokens(token_x, token_y);
    if value == false {
        return value_0.to_string();
    } else {
        value_1.to_string()
    }
}

pub fn generate_key(name: &str, val: &str) -> String {
    return format!("{}:{}", name, val);
}
