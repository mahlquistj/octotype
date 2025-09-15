tr -dc 'a-zA-Z0-9' < /dev/urandom | head -c $(($1 * $2)) | fold -w $2
