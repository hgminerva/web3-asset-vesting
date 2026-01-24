import fs from "fs";
import csv from "csv-parser";

fs.createReadStream("./vested_address.csv")
.pipe(csv())
.on('data', function(data){
    try {
        console.log("No: "+data.No);
        console.log("Address: "+data.Address);
        console.log("Balance: "+data.Balance);
        //perform the operation
    }
    catch(err) {
        //error handler
    }
})
.on('end',function(){
    //some final operation
}); 