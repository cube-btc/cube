/// RPC holder.
#[derive(Clone)]
pub struct BitcoinRPCHolder {
    url: String,
    user: String,
    password: String,
}

impl BitcoinRPCHolder {
    pub fn new(url: String, user: String, password: String) -> BitcoinRPCHolder {
        BitcoinRPCHolder {
            url,
            user,
            password,
        }
    }

    pub fn url(&self) -> String {
        self.url.clone()
    }

    pub fn user(&self) -> String {
        self.user.clone()
    }

    pub fn password(&self) -> String {
        self.password.clone()
    }
}
