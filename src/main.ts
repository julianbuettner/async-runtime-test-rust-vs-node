import { Pool } from 'pg';
const express = require('express');

const app = express();
const port = 8081;
const HASH_COST = 5_000_000;

const pool = new Pool({
  user: 'async',
  host: 'localhost',
  database: 'users',
  password: 'async',
  port: 5434,
});


function expensiveHash(input: number): number {
  let acc = 123;
  for (let x = 1; x <= HASH_COST; x++) {
    acc = (acc + x + input) % 99999;
  }
  return acc
}

const getUsers = async () => {
  const client = await pool.connect();
  try {
    const result = await client.query('SELECT id, name, hash_basis FROM users');
    return result.rows;
  } finally {
    client.release();
  }
};

app.get('/list', async (req: any, res: any) => {
  const users = await getUsers();
  const hashed_users = users.map(user => ({
    id: user.id,
    name: user.name,
    expensive_hash: expensiveHash(user.hash_basis)
  }))
  res.send(hashed_users);
});

app.listen(port, () => {
  console.log(`Example app listening at http://localhost:${port}`);
});
