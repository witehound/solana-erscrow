pub fn calculate_amount_totransfer(deal_amount: u64, commision_rate: u64) -> (u64, u64) {
    let commition_amount = deal_amount.checked_mul(commision_rate.into()).unwrap() / 100;
    let reciever_amount = deal_amount - commition_amount;

    return (reciever_amount, commition_amount);
}
