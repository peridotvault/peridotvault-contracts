pub fn check_entitlement(current_expiry: i64, incoming_expiry: i64) -> i64 {
    // 0 is permanent license.
    if current_expiry == 0 {
        return 0;
    }
    if incoming_expiry == 0 {
        return 0;
    }
    // Otherwise, both are temporary. Preserve the later expiry.
    if incoming_expiry > current_expiry {
        incoming_expiry
    } else {
        current_expiry
    }
}
