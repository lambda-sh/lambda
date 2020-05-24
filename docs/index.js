const express = require('express');
const app = express();
const port = process.env.PORT || 3000;

app.use('/', express.static('static/html'));

app.listen(
  port, () => console.log(`The app is listening on http://localhost:${port}`));
