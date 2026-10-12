import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {test} from 'node:test';
const valid={GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_REF:'refs/heads/main',ACTIONS_ID_TOKEN_REQUEST_URL:'https://fixture.invalid',ACTIONS_ID_TOKEN_REQUEST_TOKEN:'fixture',NODE_AUTH_TOKEN:'',NPM_TOKEN:''};
for(const [name,override,message] of [
 ['pull request',{GITHUB_EVENT_NAME:'pull_request'},'workflow_dispatch'],
 ['non-main ref',{GITHUB_REF:'refs/heads/release'},'refs/heads/main'],
 ['no OIDC permission',{ACTIONS_ID_TOKEN_REQUEST_URL:''},'GitHub OIDC permission is required'],
 ['no OIDC credential',{ACTIONS_ID_TOKEN_REQUEST_TOKEN:''},'GitHub OIDC request credential is required'],
 ['npm token fallback',{NPM_TOKEN:'fixture'},'Token-based publishing is not allowed'],
 ['node token fallback',{NODE_AUTH_TOKEN:'fixture'},'Token-based publishing is not allowed']
])test('reject '+name+' before packing or registry writes',()=>{
 const result=spawnSync(process.execPath,['scripts/reviewed-npm.mjs','publish'],{env:{...process.env,...valid,...override},encoding:'utf8',timeout:5000});
 assert.notEqual(result.status,0);
 assert.match(result.stderr,new RegExp(message));
 assert.doesNotMatch(result.stdout,/Verified|Official archive/);
});

test('an all-existing release cannot claim OIDC validation',()=>{
 const preload=`import fs from 'node:fs';import crypto from 'node:crypto';
 const review=JSON.parse(fs.readFileSync('.github/reviewed-npm.json'));
 globalThis.fetch=async url=>{
  const name=decodeURIComponent(new URL(url).pathname.slice(1));
  const item=review.packages.find(p=>p.name===name);
  if(!item)throw Error('Unexpected registry request');
  const archive='outgoing/'+name.slice(1).replace('/','-')+'-'+item.version+'.tgz';
  const integrity='sha512-'+crypto.createHash('sha512').update(fs.readFileSync(archive)).digest('base64');
  return {status:200,ok:true,json:async()=>({versions:{[item.version]:{dist:{integrity}}}})};
 };`;
 const result=spawnSync(process.execPath,['--import','data:text/javascript,'+encodeURIComponent(preload),'scripts/reviewed-npm.mjs','publish'],{env:{...process.env,...valid},encoding:'utf8',timeout:30000});
 assert.notEqual(result.status,0);
 assert.match(result.stderr,/No unpublished reviewed versions/);
 assert.doesNotMatch(result.stdout,/Official archive verified/);
});
