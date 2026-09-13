actor MarketEarthEngine {
    private var reserveMeCoin : Nat = 10_000_000_000;
    private var reserveWeCoin : Nat = 5_000_000_000;
    private let invariantFee : Nat = 30; // 0.3% fee tier

    public query func getReserves() : async (Nat, Nat) {
        return (reserveMeCoin, reserveWeCoin);
    }

    public shared func executeDualSwap(amountIn : Nat, isMeToWe : Bool) : async Result.Result<Nat, Text> {
        if (amountIn == 0) { return #err("Amount must be greater than zero"); };

        let (inputReserve, outputReserve) = if (isMeToWe) {
            (reserveMeCoin, reserveWeCoin);
        } else {
            (reserveWeCoin, reserveMeCoin);
        };

        if (inputReserve == 0 or outputReserve == 0) { return #err("Pool reserves depleted"); };

        let feeMultiplier : Nat = 10_000 - invariantFee;
        let amountInWithFee : Nat = amountIn * feeMultiplier;
        let numerator : Nat = amountInWithFee * outputReserve;
        let denominator : Nat = (inputReserve * 10_000) + amountInWithFee;

        if (denominator == 0) { return #err("Denominator zero error"); };

        let amountOut : Nat = numerator / denominator;
        if (amountOut >= outputReserve) { return #err("Insufficient liquidity for swap size"); };

        if (isMeToWe) {
            reserveMeCoin := reserveMeCoin + amountIn;
            reserveWeCoin := reserveWeCoin - amountOut;
        } else {
            reserveWeCoin := reserveWeCoin + amountIn;
            reserveMeCoin := reserveMeCoin - amountOut;
        };

        return #ok(amountOut);
    }
};
