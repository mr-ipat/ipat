'use strict';
// R9.47 Add Device UX. The only physically verified adapter in this build
// remains the fixed, pinned ZTE C320. Other devices are visible but disabled
// until a vendor-specific server adapter and authenticated tenant UI exist.
(() => {
  const get=id=>document.getElementById(id);
  const form=get('ipat-c320-enroll-form');
  if(!form)return;
  const state=get('ipat-c320-enroll-state');
  const output=get('ipat-c320-enroll-result');
  const button=get('ipat-c320-enroll-button');
  const type=get('ipat-device-type');
  const model=get('ipat-device-model');
  const protocol=get('ipat-device-protocol');
  const host=get('ipat-device-host');
  const port=get('ipat-device-port');
  const username=get('ipat-device-username');
  const password=get('ipat-c320-device-password');
  const owner=get('ipat-c320-bootstrap');
  const name=get('ipat-device-name');
  const notice=get('ipat-device-profile-notice');
  let busy=false, connectorOnline=false, enrolled=false;
  const models={
    olt:[
      ['zte_c320_lab','ZTE C320 (Lab Read-Only)',true],
      ['zte_other','ZTE - Other Model (Coming Soon)',false],
      ['cdata_olt','C-DATA OLT (Coming Soon)',false],
      ['vsol_olt','VSOL OLT (Coming Soon)',false]
    ],
    ont:[
      ['zte_ont','ZTE ONT (Coming Soon)',false],
      ['vsol_ont','VSOL ONT (Coming Soon)',false]
    ],
    router:[
      ['mikrotik_router','MikroTik RouterOS (Coming Soon)',false]
    ],
    other:[
      ['other_device','Custom Device (Coming Soon)',false]
    ]
  };
  const exactProfile=()=>type.value==='olt' && model.value==='zte_c320_lab' && protocol.value==='ssh';
  function redrawModels(){
    model.replaceChildren();
    for(const [id,label] of models[type.value]||[]){
      const option=document.createElement('option');
      option.value=id;
      option.textContent=label;
      // Deliberately selectable to explain which adapters are pending.
      model.append(option);
    }
    validateProfile();
  }
  function validateProfile(){
    const available=exactProfile();
    const defaultC320=host.value.trim()==='10.10.13.233'
      && port.value==='321' && username.value.trim()==='zte';
    notice.textContent=!available
      ? 'Adapter not available yet. This device cannot be connected or marked Adopted.'
      : !defaultC320
        ? 'Only the pinned lab C320 endpoint is enabled. Custom IP, port, username and additional devices require verified server-side profiles.'
        : 'ZTE C320 lab: direct pinned SSH, encrypted server-side password, read-only access after a successful live card check.';
    notice.className='ipat-profile-info'+(available&&defaultC320?'':' ipat-profile-info--warning');
    button.disabled=busy || enrolled || !connectorOnline || !available || !defaultC320;
  }
  function setState(message){state.textContent=message;}
  function update(v){
    if(v.target!=='DEV-01'||v.model!=='C320'||v.production_adopted!==false
      || v.physical_writes_enabled!==false
      || typeof v.connector_online!=='boolean'
      || typeof v.credentials_enrolled!=='boolean')throw Error('Invalid connection status');
    connectorOnline=v.connector_online;
    enrolled=v.credentials_enrolled;
    if(enrolled){
      setState('CREDENTIALS SAVED');
      form.hidden=true;
      output.textContent='Device credentials stored. Connection health is shown in the device status indicator. Use the read-only actions above to refresh inventory.';
    }else{
      form.hidden=false;
      setState(connectorOnline?'READY TO CONNECT':'CONNECTOR OFFLINE');
      validateProfile();
    }
  }
  async function refresh(){
    try{
      const response=await fetch('/lab/c320-owner-connection',{cache:'no-store',credentials:'omit'});
      if(!response.ok)throw Error('Connection API unavailable');
      update(await response.json());
    }catch{
      connectorOnline=false;
      setState('CONNECTION UNAVAILABLE');
      if(!busy)output.textContent='The private device connector is unavailable; no connection or adoption status is assumed.';
      validateProfile();
    }
  }
  type.addEventListener('change',redrawModels);
  for(const field of [model,protocol,host,port,username]){
    field.addEventListener('change',validateProfile);
    field.addEventListener('input',validateProfile);
  }
  form.addEventListener('submit',async(event)=>{
    event.preventDefault();
    if(busy||button.disabled||!exactProfile())return;
    busy=true;button.disabled=true;
    output.textContent='Connecting to ZTE C320 and verifying physical card inventory...';
    const credential=password.value,bootstrap=owner.value;
    password.value='';owner.value='';
    try{
      const address=host.value.trim();
      const sshPort=Number(port.value);
      const login=username.value.trim();
      const displayName=name.value.trim();
      if(address!=='10.10.13.233'||sshPort!==321||login!=='zte'
         || !displayName||displayName.length>64||!credential||!bootstrap)
         throw Error('Check supported device profile and required fields.');
      const controller=new AbortController();
      const timer=setTimeout(()=>controller.abort(),60000);
      let response;
      try{
        response=await fetch('/lab/c320-owner-enroll',{
          method:'POST',credentials:'omit',cache:'no-store',
          headers:{'Content-Type':'application/json','X-IPAT-Demo-Only':'1'},
          body:JSON.stringify({
            device_profile:'zte_c320_lab',device_type:type.value,
            device_name:displayName,management_ip:address,
            ssh_port:sshPort,username:login,
            bootstrap_code:bootstrap,password:credential
          }),signal:controller.signal
        });
      }finally{clearTimeout(timer);}
      if(!response.ok)throw Error(response.status===403
        ? 'Owner verification or physical authentication failed. Confirm the device credentials.'
        : 'Connection failed (HTTP '+response.status+').');
      const verified=await response.json();
      if(verified.enrolled_for_read!==true||verified.production_adopted!==false
        || verified.physical_writes_enabled!==false
        || verified.adoption_state!=='READ_ONLY_CONNECTED_LAB'
        || !Number.isSafeInteger(verified.card_count)||verified.card_count<1)
        throw Error('Physical device verification failed.');
      output.textContent='CONNECTED (READ-ONLY) · '+verified.card_count
        +' active cards verified at '+verified.verified_at_utc+'.';
      await refresh();
      window.dispatchEvent(new Event('ipat-device-connection-changed'));
    }catch(error){
      output.textContent='NOT CONNECTED · '+error.message;
    }finally{
      busy=false;
      await refresh();
      validateProfile();
    }
  });
  redrawModels();
  refresh();
  setInterval(refresh,15000);
})();
