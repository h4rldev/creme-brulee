{pkgs ? import <nixpkgs> {}}:
with pkgs;
  mkShell {
    name = "creme-brulee";

    packages = [
      rustup
      openssl
      pkg-config
      curlHTTP3
    ];

    shellHook = ''
      mkdir -p .certs

      if [[ ! -f .certs/localhost.crt ]] || [[ ! -f .certs/localhost.key ]]; then
        openssl req -x509 -newkey rsa:2048 -nodes -keyout ./.certs/localhost.key \
            -out ./.certs/localhost.crt -days 365 -subj "/CN=localhost" \
            -addext "subjectAltName = DNS:localhost"
      fi

      local EXPIRATION_DATE=$(openssl x509 -in ./.certs/localhost.crt -noout -enddate | cut -d= -f2)
      echo "Certificate expires on $EXPIRATION_DATE"
      local EXPIRATION_EPOCH=$(date -d "$EXPIRATION_DATE" +%s)
      local NOW_EPOCH=$(date +%s)

      if [[ $EXPIRATION_EPOCH -lt $NOW_EPOCH ]]; then
        echo "Certificate expired on $EXPIRATION_DATE"
        echo "Renewing certificate..."

        # Generate new CSR
        openssl x509 -x509toreq -in .certs/localhost.crt -out .certs/csr.pem -signkey .certs/localhost.key

        # Generate new certificate with extended validity
        openssl x509 -req -days 365 -in .certs/csr.pem -signkey .certs/localhost.key -out .certs/localhost.crt

        # Clean up CSR
        rm .certs/csr.pem
      fi

      cat .certs/localhost.crt /etc/ssl/certs/ca-certificates.crt > .certs/ca-bundle.crt

      export SSL_CERT_FILE=.certs/ca-bundle.crt
      export SSL_CERT_DIR=.certs

      echo "Temporary certificate trust enabled"
    '';
  }
