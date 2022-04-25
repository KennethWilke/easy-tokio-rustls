COUNTRY="US"
STATE="State"
LOCATION="City"
ORGANIZATION="Selfie Signers"
NAME="self.signed.domain"
SUBJECT="/C=$COUNTRY/ST=$STATE/L=$LOCATION/O=$ORGANIZATION/CN=$NAME"

openssl req -newkey rsa:2048 -nodes -subj "$SUBJECT" \
    -keyout privkey.pem -x509 -days 365 -out cert.pem
