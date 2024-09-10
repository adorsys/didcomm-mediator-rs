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

##### 2. Mediate Request
