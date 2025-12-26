import { pbkdf2Sync, randomBytes } from "node:crypto";

const email = process.argv[2];
const password = process.argv[3];

if (!email || !password) {
  console.error("usage: node scripts/create-user.mjs <email> <password>");
  process.exit(1);
}

const salt = randomBytes(16);
const iters = 200_000;
const hash = pbkdf2Sync(password, salt, iters, 32, "sha256");
const userId = `usr_${crypto.randomUUID()}`;

console.log("-- Insert user into D1:");
console.log(
  `INSERT INTO users (id, email, password_hash, password_salt, password_iters, created_at)\n` +
    `VALUES ('${userId}', '${email}', '${hash.toString("base64")}', '${salt.toString("base64")}', ${iters}, '${new Date().toISOString()}');`
);
