var fs = require('fs');
var cp = require('child_process');
var dir = fs.existsSync('src-ui') ? 'src-ui' : '../src-ui';
process.chdir(dir);
cp.execSync('pnpm build', {stdio: 'inherit'});