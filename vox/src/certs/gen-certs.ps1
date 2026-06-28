New-Item -ItemType Directory -Force newcerts | Out-Null
New-Item -ItemType File -Force index.txt | Out-Null
[System.IO.File]::WriteAllText("serial", "1000`n")
[System.IO.File]::WriteAllText("crlnumber", "1000`n")

# CA
openssl genrsa -out ca.key 4096
openssl req -x509 `
    -new `
    -nodes `
    -key ca.key `
    -sha256 `
    -days 3650 `
    -out ca.pem `
    -config openssl.cnf `
    -extensions v3_ca

# Server key
openssl genrsa -out server.key 2048

# CSR
openssl req `
    -new `
    -key server.key `
    -out server.csr `
    -subj "/CN=doh.local"

# Sign
openssl ca `
    -config openssl.cnf `
    -extensions server_cert `
    -days 3650 `
    -notext `
    -batch `
    -in server.csr `
    -out server.pem

# Empty CRL
openssl ca `
    -config openssl.cnf `
    -gencrl `
    -out crl.pem
