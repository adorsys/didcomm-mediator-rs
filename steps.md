create global env file to set environment variables
- keystore created ```(susceptible to be mediator's secret)```
- did.json file created ```(did document)```
- oob invitation.txt and qrcode.txt created

##### 1. Get out of  band invitation
###### Routes
- ```/``` ```/oob_qr```: get oob QR-code invitation
- ```/oob_url```: get oob invitation url
- ```/.well-known/did.json```: get did document ```(Bad Request)```
- ```/about```: about 

![output](../didcomm-mediator-rs/docs/oob.png)
##### 2. Mediate Request
- make post request at ```localhost:3000/mediate``` with encrypted mediate request body and set request content-type header to ```application/didcomm-encrypted+json```.
    issues, on mediation grant no ```routing_did``` returned.<br>
**Sample message**
```json
{"protected":"eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIiwiYWxnIjoiRUNESC0xUFUrQTI1NktXIiwiZW5jIjoiQTI1NkNCQy1IUzUxMiIsInNraWQiOiJkaWQ6a2V5Ono2TWtmeVRSRWpUeFE4aFV3U3dCUGVESGYzdVBMM3FDalNTdU5Qd3N5TXBXVUdINyN6NkxTYnVVWFdTZ1BmcGlEQmpVSzZFN3lpQ0tNTjJlS0pzWG41YjU1WmdxR3o2TXIiLCJhcHUiOiJaR2xrT210bGVUcDZOazFyWm5sVVVrVnFWSGhST0doVmQxTjNRbEJsUkVobU0zVlFURE54UTJwVFUzVk9VSGR6ZVUxd1YxVkhTRGNqZWpaTVUySjFWVmhYVTJkUVpuQnBSRUpxVlVzMlJUZDVhVU5MVFU0eVpVdEtjMWh1TldJMU5WcG5jVWQ2TmsxeSIsImFwdiI6Il9jSm9kT2NFdjRFSVY1ZW96U2N3M19wZFN2OVNJSnRsY0FIYVRzWW1YZ2MiLCJlcGsiOnsiY3J2IjoiWDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6IjlQaVN6UkNNcEZnSjRmaHYwNkd2Q3BMcElNOHFHdThmZnRYdHVtcWZGV3cifX0","recipients":[{"header":{"kid":"did:web:alice-mediator.com:alice_mediator_pub#keys-3"},"encrypted_key":"QJHtLkdama14uLNSDUthRY2pGZC1YDuMwk4ImCJhkwqcfdzN5-NUS3dPvUSl8viKL6mGn-XVI3dkH_FKlJUf4Jfng4f_U-Sr"}],"iv":"kUgkAA_y_C2u_Sg-7PKyYg","ciphertext":"H22gr7s4B18KppAYbOe_KV0dDfbEqImJqVwBDdx48ODbRuquBhAcIF8UNMnzkf4HBC7ngkoCVpQHj4HG4n37cZnogDEBdmpipp-HYuuwMM7sfFkG2sc_MwQGndBx9gAXAcEzruEq8byBNCV2us6ixQcIQWue57SQenlNPUsmLzC5sbpt06lj34tv7yrWU3B4c4L9LYzHUr0jYbdCrS9oULJSp7c1axvmU_gcIAMfTROeOx8a0sxFnrXziGANnxa2WIzU952ZWLJ553-W15h3KBcZcnfliN3gnvO_Y9ny4nwAei9xZFmvQPCqm-YpRw02TM9T_8VGYzxiqJOf9M8vwiXXGqgzT_U-zRvtKR8Uuf4_Wz5rXDsyik12UwP_thjUL6_fwg8zVbLdDyt12rohOSghgrdbPzv-migs5gVKAro","tag":"I7CdFdAOXf-WOiRWulC2QirGRxSX9xEzqan2I3BkXiM"}
```

![output](../didcomm-mediator-rs/docs/mediate_request.png)