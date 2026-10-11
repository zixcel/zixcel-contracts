import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync} from 'node:child_process';
const root=process.cwd(), registry='https://registry.npmjs.org';
const review=JSON.parse(fs.readFileSync(path.join(root,'.github/reviewed-npm.json')));
async function registryVersion(name,version){
 const response=await fetch(`${registry}/${name.replace('/','%2f')}?verify=${Date.now()}`,{headers:{'cache-control':'no-cache','accept':'application/vnd.npm.install-v1+json'}});
 if(response.status===404)return null;
 assert.ok(response.ok,`Registry metadata failed: ${response.status}`);
 return (await response.json()).versions?.[version]??null;
}
const mode=process.argv[2]??'prepare';
assert.ok(['prepare','publish','verify-registry'].includes(mode));
assert.equal(execFileSync('git',['remote','get-url','origin'],{encoding:'utf8'}).trim().replace(/\.git$/, ''),`https://github.com/${review.repository}`);
if(process.env.GITHUB_REPOSITORY)assert.equal(process.env.GITHUB_REPOSITORY,review.repository);
const output=path.join(root,'outgoing');fs.mkdirSync(output,{recursive:true});
const packages=[];
for(const item of review.packages){
 const directory=path.resolve(root,item.directory);assert.ok(directory===root||directory.startsWith(root+path.sep));
 const manifest=JSON.parse(fs.readFileSync(path.join(directory,'package.json')));
 assert.equal(manifest.name,item.name);assert.equal(manifest.version,item.version);
 assert.equal(manifest.repository.url,`git+https://github.com/${review.repository}.git`);
 if(item.directory!=='.')assert.equal(manifest.repository.directory,item.directory);
 for(const section of ['dependencies','optionalDependencies','peerDependencies','devDependencies']){
  for(const [name,version] of Object.entries(manifest[section]??{}))assert.ok(!/^(?:file:|link:|workspace:|git|https?:|\.\.?\/)/.test(version),`Non-registry dependency: ${name}`);
 }
 const data=JSON.parse(execFileSync('npm',['pack','--ignore-scripts','--json','--pack-destination',output],{cwd:directory,encoding:'utf8',stdio:['ignore','pipe','pipe']}));
 const rows=Array.isArray(data)?data:Object.values(data);assert.equal(rows.length,1);
 const packed=rows[0];assert.equal(packed.name,item.name);assert.equal(packed.version,item.version);
 assert.equal(path.basename(packed.filename),packed.filename);
 for(const file of ['LICENSE','NOTICE','README.md'])assert.ok(packed.files.some(entry=>entry.path===file),`${item.name}: missing ${file}`);
 assert.ok(!packed.files.some(entry=>/^(?:test|tests|node_modules)\//.test(entry.path)));
 const archive=path.join(output,packed.filename), bytes=fs.readFileSync(archive);
 assert.equal(crypto.createHash('sha256').update(bytes).digest('hex'),item.sha256,`${item.name}: archive differs from review`);
 packages.push({...item,archive,integrity:'sha512-'+crypto.createHash('sha512').update(bytes).digest('base64')});
}
console.log(`Verified ${packages.length} reviewed ${review.repository} archives`);
if(mode==='publish'){
 assert.equal(process.env.GITHUB_EVENT_NAME,'workflow_dispatch');
 assert.equal(process.env.GITHUB_REF,'refs/heads/main');
 assert.ok(process.env.ACTIONS_ID_TOKEN_REQUEST_URL,'GitHub OIDC permission is required');
 const pending=[];
 for(const item of packages){
  const existing=await registryVersion(item.name,item.version);
  if(!existing)pending.push(item);
  else assert.equal(existing.dist.integrity,item.integrity,`${item.name}: existing registry content differs`);
 }
 for(const item of pending){
  execFileSync('npm',['publish',item.archive,'--access','public','--ignore-scripts','--provenance','--registry='+registry,'--loglevel=warn'],{cwd:root,stdio:'inherit'});
 }
}
if(['publish','verify-registry'].includes(mode)){
 const outstanding=new Map(packages.map(item=>[item.name,item]));
 const deadline=Date.now()+8*60*1000;
 while(outstanding.size&&Date.now()<deadline){
  for(const [name,item] of outstanding){
   const metadata=await registryVersion(name,item.version);if(!metadata)continue;
   assert.equal(metadata.dist.integrity,item.integrity,`${name}: registry content differs`);
   assert.equal(new URL(metadata.dist.tarball).origin,registry);
   const official=await fetch(metadata.dist.tarball,{headers:{'cache-control':'no-cache'}});
   if(official.status===404)continue;assert.ok(official.ok,`${name}: archive unavailable (${official.status})`);
   assert.equal(crypto.createHash('sha256').update(Buffer.from(await official.arrayBuffer())).digest('hex'),item.sha256,`${name}: official archive differs`);
   outstanding.delete(name);console.log(`Official archive verified: ${name}@${item.version}`);
  }
  if(outstanding.size)await new Promise(resolve=>setTimeout(resolve,5000));
 }
 assert.equal(outstanding.size,0,`Registry propagation pending: ${[...outstanding.keys()].join(', ')}`);
}
