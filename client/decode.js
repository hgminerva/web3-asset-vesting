export function decode(data) {
    if (!data || data.length < 2) {
        throw new Error("Invalid event data format");
    }

    const [address, raw] = data;

    const bytes = raw.toU8a();
    const operatorHash = bytes.slice(0, 32);    // topic in payload
    const payload = bytes.slice(32);            // event payload

    //console.log(payload);

    const errorMap = [
        "Error::BadOrigin",
        "Error::VestedBalanceAlreadyExist",
        "Error::VestedBalanceNotFound",
        "Error::VestedBalanceScheduleNotFound",
        "Error::VestedBalanceScheduleNotLiquid",
        "Error::VestedBalanceScheduleNotRequested",
    ]; 

    const successMap = [
        "Success::VestingSetupSuccess",
        "Success::VestedBalanceAdded",
        "Success::VestedBalanceRemoved",
        "Success::VestedBalanceScheduleThawed",
        "Success::VestedBalanceScheduleRequested",
        "Success::VestedBalanceScheduleApproved",
    ];     

    if (payload[2] === 0) {
        return successMap[payload[3]];
    } else if (payload[2] === 1) {
        return errorMap[payload[3]];
    } else {
        throw new Error("Invalid event payload");
    }    
}