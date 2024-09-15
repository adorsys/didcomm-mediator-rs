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
- make post request at ```localhost:3000/mediate``` with encrypted mediate request body and set request content-type header to ```application/didcomm-encrypted+json```.<br>
* issues, on mediation grant no ```routing_did``` returned.<br>
**Sample message**
```json
{"protected":"eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIiwiYWxnIjoiRUNESC0xUFUrQTI1NktXIiwiZW5jIjoiQTI1NkNCQy1IUzUxMiIsInNraWQiOiJkaWQ6a2V5Ono2TWtmeVRSRWpUeFE4aFV3U3dCUGVESGYzdVBMM3FDalNTdU5Qd3N5TXBXVUdINyN6NkxTYnVVWFdTZ1BmcGlEQmpVSzZFN3lpQ0tNTjJlS0pzWG41YjU1WmdxR3o2TXIiLCJhcHUiOiJaR2xrT210bGVUcDZOazFyWm5sVVVrVnFWSGhST0doVmQxTjNRbEJsUkVobU0zVlFURE54UTJwVFUzVk9VSGR6ZVUxd1YxVkhTRGNqZWpaTVUySjFWVmhYVTJkUVpuQnBSRUpxVlVzMlJUZDVhVU5MVFU0eVpVdEtjMWh1TldJMU5WcG5jVWQ2TmsxeSIsImFwdiI6Il9jSm9kT2NFdjRFSVY1ZW96U2N3M19wZFN2OVNJSnRsY0FIYVRzWW1YZ2MiLCJlcGsiOnsiY3J2IjoiWDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6IjlQaVN6UkNNcEZnSjRmaHYwNkd2Q3BMcElNOHFHdThmZnRYdHVtcWZGV3cifX0","recipients":[{"header":{"kid":"did:web:alice-mediator.com:alice_mediator_pub#keys-3"},"encrypted_key":"QJHtLkdama14uLNSDUthRY2pGZC1YDuMwk4ImCJhkwqcfdzN5-NUS3dPvUSl8viKL6mGn-XVI3dkH_FKlJUf4Jfng4f_U-Sr"}],"iv":"kUgkAA_y_C2u_Sg-7PKyYg","ciphertext":"H22gr7s4B18KppAYbOe_KV0dDfbEqImJqVwBDdx48ODbRuquBhAcIF8UNMnzkf4HBC7ngkoCVpQHj4HG4n37cZnogDEBdmpipp-HYuuwMM7sfFkG2sc_MwQGndBx9gAXAcEzruEq8byBNCV2us6ixQcIQWue57SQenlNPUsmLzC5sbpt06lj34tv7yrWU3B4c4L9LYzHUr0jYbdCrS9oULJSp7c1axvmU_gcIAMfTROeOx8a0sxFnrXziGANnxa2WIzU952ZWLJ553-W15h3KBcZcnfliN3gnvO_Y9ny4nwAei9xZFmvQPCqm-YpRw02TM9T_8VGYzxiqJOf9M8vwiXXGqgzT_U-zRvtKR8Uuf4_Wz5rXDsyik12UwP_thjUL6_fwg8zVbLdDyt12rohOSghgrdbPzv-migs5gVKAro","tag":"I7CdFdAOXf-WOiRWulC2QirGRxSX9xEzqan2I3BkXiM"}
```

![output](../didcomm-mediator-rs/docs/mediate_request.png)

##### 3. keylist Update 
- make post request at ```localhost:3000/mediate``` with encrypted mediate request body and set request content-type header to ```application/didcomm-encrypted+json```.
**Sample message**
```json
{"protected":"eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIiwiYWxnIjoiRUNESC0xUFUrQTI1NktXIiwiZW5jIjoiQTI1NkNCQy1IUzUxMiIsInNraWQiOiJkaWQ6a2V5Ono2TWtmeVRSRWpUeFE4aFV3U3dCUGVESGYzdVBMM3FDalNTdU5Qd3N5TXBXVUdINyN6NkxTYnVVWFdTZ1BmcGlEQmpVSzZFN3lpQ0tNTjJlS0pzWG41YjU1WmdxR3o2TXIiLCJhcHUiOiJaR2xrT210bGVUcDZOazFyWm5sVVVrVnFWSGhST0doVmQxTjNRbEJsUkVobU0zVlFURE54UTJwVFUzVk9VSGR6ZVUxd1YxVkhTRGNqZWpaTVUySjFWVmhYVTJkUVpuQnBSRUpxVlVzMlJUZDVhVU5MVFU0eVpVdEtjMWh1TldJMU5WcG5jVWQ2TmsxeSIsImFwdiI6Il9jSm9kT2NFdjRFSVY1ZW96U2N3M19wZFN2OVNJSnRsY0FIYVRzWW1YZ2MiLCJlcGsiOnsiY3J2IjoiWDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6IkhvcjNuUGw5aEUxYUt3VnkxZHlwOWdZUkNHWmxkMXRCTjR5YVJ4MTJfaEUifX0","recipients":[{"header":{"kid":"did:web:alice-mediator.com:alice_mediator_pub#keys-3"},"encrypted_key":"cPT4rIA_suepUB6quB6YsKclTNuU3cFWoZrZQykZ4jzZGRSDGhwrfulvR0Lg3_baZPsLumYb7jvPU39AX5AXA1GT_CHOQcf7"}],"iv":"mV1qKll1-4hyTv0dPv3ncA","ciphertext":"Ja0k4giXmw6vy2Bo2WnzdESFFvXd-MnaVhb6GtaCk1VoBEcd-QFduPO4FAxBNHQuLsQ3daWeGiBM788T9s4dAKre7FUB-l8yJGTXZjBvK_8GSD4G5IOFqKfTZjMDoI4E3tX2haxFKKTXs9W2zEae3C9FRN3onKKrlVeqBhuYX3o-9kRhVp8YCI4df0WzUX3_1BhX4oHoyFfOc1SO0A0hpcRyr1PMEPeZDza6gxwzMb5AFPsomyhV8eJ0BIblYIRvILi_lCpjd3Q8ht6mgaYYyQlekQwYuUx11BdbX7SvN_w__uAlB9ft9ehogot-A2B7Pinv2PDF4C43T4PAsNU9N-Yof3T-HdIsR3EB3ucNPqHcDRoceGPMzdP5RwH-mY_kCuqNj7ybEpz3hUNnSDxEM8nuOp2zgNxTjh1io53M5cTVw1Ov4_fcFMpzNlxk8gyCOp3w0gXqEvSqkRNtwdLHCZk4UC_a2sUQbYz29_o4JdtwH_uJzC8krHLyhhDZafxRp_61QHsRR5UjBYipU7QCH5shroYQZlwlUFtJFR11vlTTBaYFwUhvPzMMOHazx8d4i1JxkhTGsGo6VirzOuue2Wb59lCZNqdNP7UcxqlOGxNa_gMH6f3Xy3U5a9ppEe2u","tag":"F2peaL3EweEHyqVmlF8aNld_oIZF5bBfvltF7RE67X4"}
```
![Output](../didcomm-mediator-rs/docs/keylist_update.png)

##### 4. keylist query
- make post request at ```localhost:3000/mediate``` with encrypted mediate request body and set request content-type header to ```application/didcomm-encrypted+json```.
**Sample message**
```json
{"protected":"eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIiwiYWxnIjoiRUNESC0xUFUrQTI1NktXIiwiZW5jIjoiQTI1NkNCQy1IUzUxMiIsInNraWQiOiJkaWQ6a2V5Ono2TWtmeVRSRWpUeFE4aFV3U3dCUGVESGYzdVBMM3FDalNTdU5Qd3N5TXBXVUdINyN6NkxTYnVVWFdTZ1BmcGlEQmpVSzZFN3lpQ0tNTjJlS0pzWG41YjU1WmdxR3o2TXIiLCJhcHUiOiJaR2xrT210bGVUcDZOazFyWm5sVVVrVnFWSGhST0doVmQxTjNRbEJsUkVobU0zVlFURE54UTJwVFUzVk9VSGR6ZVUxd1YxVkhTRGNqZWpaTVUySjFWVmhYVTJkUVpuQnBSRUpxVlVzMlJUZDVhVU5MVFU0eVpVdEtjMWh1TldJMU5WcG5jVWQ2TmsxeSIsImFwdiI6Il9jSm9kT2NFdjRFSVY1ZW96U2N3M19wZFN2OVNJSnRsY0FIYVRzWW1YZ2MiLCJlcGsiOnsiY3J2IjoiWDI1NTE5Iiwia3R5IjoiT0tQIiwieCI6ImxraHRZcks3bDJHdHNKeW9QWDZadDItc1ZjclVQSXVRSlE4dGhCMEMzZ1UifX0","recipients":[{"header":{"kid":"did:web:alice-mediator.com:alice_mediator_pub#keys-3"},"encrypted_key":"O75AvmFUF_AtxygDBnfAE8Wc2ncyK36cJ0DfkRXw-NFBRGTFRuG9yM6kk9eJ2TdxaTLhj2t8fU7jZ93qB6Fyfj9Zwh-dpy0G"}],"iv":"zbveAaDIwekPjpOc44aRvw","ciphertext":"sdoGXQ_4ve5yPzI3el69tl1VWaGufKyKqH_4QSnw_BZPIHBp8ZEWzfc_3aE8eLyU7P3OpVlAsqbw2ZUlNDGxjo-MJr7Qc_3Q6OWeZl2yQmNpqLrEiHeHZ8MeasrCeAzk6_wuESthi8TJEssJYsmzFJlDJpZdvK_IRYTf9AmvbFMEwPud-h3RUs5a-74zmMeBCw83ctwE2xh_-VMIq_ijqvCVD6Ol7yMX56ncMraRAOaBC-8wVz58z5gH_OtfRtsPwxDb6bB_6Htnl51GDcgaDy0T7CkJ2KwuMANF0wpZd6IOOASzvUal_J5uca97aewAjv3geZhpLmVo_6rlPwTgqudGBldPw0px_-w3NTeb8ww","tag":"yf7t83i9VupdaiVQ6dZgcw-x6G5736eBhe99Dj84eyE"}
```
![output](../didcomm-mediator-rs/docs/keylistQuery.png)
- issues ***keylist*** return to mediator and not sender.