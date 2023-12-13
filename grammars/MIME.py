ctx.rule("START","{MIME}")

ctx.rule("HEADER",u"{FROM}\n{TO}\n{SUBJECT}\n{MIME_VERSION}\n{CONTENT_TYPE}")
ctx.rule("FROM","From: {ADDRESS}")
ctx.rule("TO","To: {ADDRESS}")
ctx.regex("SUBJECT","[a-zA-Z0-9]+")
ctx.rule("MIME_VERSION","MIME-Version: 1.0")

ctx.rule("CONTENT_TYPE","Content-Type: {TYPE}")
ctx.rule("CONTENT_TRANSFER_ENCODING","Content-Transfer-Encoding: {TRANSFER_ENCODING}")

ctx.rule("TYPE","text/plain; charset=\"utf-8\"")
ctx.rule("TYPE","text/html; charset=\"utf-8\"")
ctx.rule("TYPE","text/css; charset=\"utf-8\"")
ctx.rule("TYPE","text/javascript; charset=\"utf-8\"")
ctx.rule("TYPE","text/xml; charset=\"utf-8\"")

ctx.rule("TYPE","application/json;")
ctx.rule("TYPE","application/xml;charset=\"utf-8\"")

ctx.rule("TYPE","image/jpeg;")
ctx.rule("TYPE","image/png;")

ctx.rule("TYPE","audio/mpeg;")
ctx.rule("TYPE","video/mp4;")

ctx.rule("TRANSFER_ENCODING", "7bit")
ctx.rule("TRANSFER_ENCODING", "binary")
ctx.rule("TRANSFER_ENCODING", "quoted-printable")
ctx.rule("TRANSFER_ENCODING", "base64")

ctx.rule("ADDRESS", "{USER}@{DOMAIN}.com")
ctx.regex("USER","[a-zA-Z0-9]+")
ctx.regex("DOMAIN","[a-zA-Z0-9]+")

# This generates a valid Message Authentication Code for a given header and body
ctx.script("MIME", ["HEADER", "BODY"], lambda header, body, hashlib=__import__('hashlib'):"{}\nX-MAC:{}\n\n{}".format(header.decode("utf-8"), hashlib.sha256(header + b'\n\n' + body).hexdigest(), body.decode("utf-8")))

ctx.regex("BODY","[a-zA-Z0-9]+")
