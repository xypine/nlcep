const fs = require("fs"); // replace with import fs from 'fs'; if you need
const packageFileContent = fs.readFileSync("./pkg/package.json", "utf-8");
const packageJSON = JSON.parse(packageFileContent);
packageJSON.type = "module";
packageJSON.main = packageJSON.module;
fs.writeFileSync("./pkg/package.json", JSON.stringify(packageJSON, null, 2), "utf-8");
