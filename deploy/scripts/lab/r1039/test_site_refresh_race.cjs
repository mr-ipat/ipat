'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
(async()=>{
 const src=fs.readFileSync(path.resolve(__dirname,'../../../../web/console/tenant/app.js'),'utf8');
 const start=src.indexOf(' async function refreshSites(after){');
 const end=src.indexOf(' async function loadCatalog()',start);
 assert(start>=0&&end>start,'use the actual commercial UI implementation');
 const production=src.slice(start,end);
 assert.match(production,/generation=\+\+state\.sitesFetchGeneration/);
 assert.match(production,/if\(generation!==state\.sitesFetchGeneration\)return/);
 const pending=[],choice=[];
 const dom=new Map();
 const get=id=>{
  if(!dom.has(id))dom.set(id,{value:'',hidden:false,textContent:'',replaceChildren(){},append(){}});
  return dom.get(id);
 };
 const state={sitesFetchGeneration:0,sites:[],pendingSite:null,next:null};
 const context={state,$:get,api:()=>new Promise(resolve=>pending.push(resolve)),
  fillSiteChoices:(id,val)=>{choice.push([id,val]);get(id).value=val||''},
  document:{createElement:()=>({append(){}})},cell:()=>({})};
 vm.createContext(context);
 const refresh=vm.runInContext(production+'; refreshSites',context);
 const old=refresh();
 const current=refresh();
 assert.equal(pending.length,2);
 pending[1]({sites:[],next_after:null});
 await current;
 const choices=choice.length;
 pending[0]({sites:[{code:'STALE',display_name:'Old response'}],next_after:null});
 await old;
 assert.equal(state.sites.length,0,'older response must not mutate newer inventory');
 assert.equal(choice.length,choices,'older response must not overwrite Site selection');
 get('device-site').value='CURRENT_OPERATOR_CHOICE';
 const next=refresh();
 get('device-site').value='NEW_OPERATOR_CHOICE';
 pending[2]({sites:[],next_after:null});
 await next;
 assert.equal(choice.findLast(x=>x[0]==='device-site')[1],'NEW_OPERATOR_CHOICE',
  'newest refresh must capture selection after GET, not before');
 console.log('R1039_SITE_FETCH_OUT_OF_ORDER_AND_LIVE_SELECTION_VM=PASS');
})().catch(error=>{console.error(error);process.exitCode=1});
