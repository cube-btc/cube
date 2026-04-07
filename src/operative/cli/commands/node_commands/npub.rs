use crate::transmutative::key::KeyHolder;

// npub
pub async fn npub_command(key_holder: &KeyHolder) {
    println!("{}", key_holder.npub());
}
