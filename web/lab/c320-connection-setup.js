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
  const saveStage=get('ipat-device-save-stage');
  const connectStage=get('ipat-device-connect-stage');
  const diagnostic=get('ipat-device-diagnostic');
  let busy=false, connectorOnline=false, enrolled=false;
  let waitingForOwner=false, networkChecked=false;
  const showStage=(saved,connection)=>{if(saved)saveStage.textContent=saved;if(connection)connectStage.textContent=connection;};
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
      setState(v.device_status==='CONNECTED'?'CONNECTED':'CREDENTIALS SAVED');
      showStage('Saved to Device List',v.device_status==='CONNECTED'?'Connected · Read-only':'Credentials stored · Checking device');
      form.hidden=true;
      if(v.device_status==='CONNECTED') output.textContent='Device connected. Use the read-only inventory and diagnostics panels.';
    }else{
      form.hidden=false;
      setState(connectorOnline?'READY TO CONNECT':'CONNECTOR OFFLINE');
      if(v.draft_saved===true){
        if(!busy)showStage('Saved to Device List',waitingForOwner?'Pending · One-time lab verification required':connectorOnline?'Pending · Authentication required':'Pending · Connector unavailable');
        if(!networkChecked && connectorOnline){
          networkChecked=true;
          void probe().then(message=>{diagnostic.textContent=message;});
        }
      }else showStage('Not Saved',connectorOnline?'Awaiting device registration':'Connector unavailable');
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
      showStage(null,'Unknown · Connector unavailable');
      diagnostic.textContent='CONNECTOR_UNAVAILABLE: IPAT could not reach its private device worker. Check the server service, not the OLT password.';
      if(!busy)output.textContent='Device connection is not verified. Any previously saved device remains in Device List.';
      validateProfile();
    }
  }
  type.addEventListener('change',redrawModels);
  for(const field of [model,protocol,host,port,username]){
    field.addEventListener('change',validateProfile);
    field.addEventListener('input',validateProfile);
  }
  const security=form.querySelector('.ipat-device-security');
  const diagnostics={
    OWNER_VERIFICATION_FAILED:'Owner verification failed or rate-limited. Open Advanced Security and check your one-time owner code.',
    SSH_HANDSHAKE_OR_AUTH_METHOD:'SSH handshake or authentication method is unsupported. Verify the SSH service and device crypto profile.',
    SSH_AUTH_FAILED_OR_UNKNOWN_PROMPT:'SSH login was not verified. Check username/password and device CLI prompt. The password is not stored.',
    DEVICE_CLI_RESPONSE_UNSUPPORTED:'SSH session opened, but the actual device CLI response does not match the tested adapter.',
    ALREADY_ENROLLED:'Device credentials were already enrolled. Refresh Device List instead of creating a duplicate.',
    DEVICE_CONNECTION_FAILED:'Device connection failed; check the network diagnostic and server connector.'
  };
  async function saveDraft(input){
    const response=await fetch('/lab/c320-owner-save-draft',{
      method:'POST',credentials:'omit',cache:'no-store',
      headers:{'Content-Type':'application/json','X-IPAT-Demo-Only':'1'},
      body:JSON.stringify(input)
    });
    const data=await response.json().catch(()=>({}));
    if(!response.ok||data.saved!==true||data.adoption_state!=='DRAFT_SAVED_AWAITING_AUTH'){
      throw Error('Failed to save pending device record (HTTP '+response.status+').');
    }
    window.dispatchEvent(new Event('ipat-device-connection-changed'));
    return data;
  }
  async function probe(){
    try{
      const response=await fetch('/lab/c320-owner-network-probe',{
        cache:'no-store',credentials:'omit'
      });
      if(!response.ok)throw Error('Network diagnostic unavailable (HTTP '+response.status+').');
      const data=await response.json();
      const messages={
        TCP_REACHABLE_AUTH_NOT_TESTED:'TCP port reachable. SSH credentials and physical device identity have NOT been verified.',
        SSH_PORT_REFUSED:'Connection refused: SSH service may be disabled or configured on a different port.',
        NETWORK_UNREACHABLE:'Network unreachable: check routes, tunnel configuration and management ACL.',
        NETWORK_OR_PORT_ERROR:'Network or management port error: inspect route, firewall and SSH listener.',
        TCP_TIMEOUT_NETWORK_OR_PORT:'Connection timeout: management route, firewall and SSH port cannot be distinguished by TCP alone.'
      };
      return messages[data.probe_stage]||'Network diagnostic returned an unknown state.';
    }catch{return 'Network diagnostic could not run; the device status remains Pending.';}
  }
  form.addEventListener('submit',async(event)=>{
    event.preventDefault();
    if(busy||button.disabled||!exactProfile())return;
    busy=true;button.disabled=true;
    showStage('Saving device...','Waiting for network check');
    output.textContent='Saving device to Device List before attempting a connection...';
    const credential=password.value,bootstrap=owner.value;
    let draftSaved=false;
    try{
      const address=host.value.trim();
      const sshPort=Number(port.value);
      const login=username.value.trim();
      const displayName=name.value.trim();
      if(address!=='10.10.13.233'||sshPort!==321||login!=='zte'
         || !displayName||displayName.length>64)
        throw Error('Unsupported device profile or management endpoint. Review Management IP, SSH Port and Username.');
      const draft={device_profile:'zte_c320_lab',device_type:type.value,
        device_name:displayName,management_ip:address,ssh_port:sshPort,username:login};
      await saveDraft(draft);
      draftSaved=true;
      networkChecked=true;
      showStage('Saved to Device List','Pending · Checking management network');
      diagnostic.textContent='NETWORK_PROBE_RUNNING: Checking the configured management endpoint...';
      output.textContent='SAVED TO DEVICE LIST · Running management network diagnostic...';
      const network=await probe();
      diagnostic.textContent=network;
      if(!credential){
        showStage(null,'Pending · SSH password required');
        output.textContent='DEVICE SAVED · SSH password required before connection. '+network;
        password.focus();
        return;
      }
      if(!bootstrap){
        waitingForOwner=true;
        if(security)security.open=true;
        showStage(null,'Pending · One-time lab verification required');
        output.textContent='DEVICE SAVED · LAB_SETUP_CODE_REQUIRED: Follow the instructions in the expanded Lab Setup Verification section. Paste the code there to resume the pending connection. '+network;
        owner.focus();
        return;
      }
      if(!network.startsWith('TCP port reachable')){
        showStage(null,'Pending · Management network / SSH service');
        output.textContent='DEVICE SAVED · Connection pending. '+network;
        return;
      }
      waitingForOwner=false;
      showStage(null,'Connecting · SSH authentication and physical read');
      password.value='';owner.value='';
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
      if(!response.ok){
        const failure=await response.json().catch(()=>({}));
        const stage=typeof failure.error==='string'?failure.error:'DEVICE_CONNECTION_FAILED';
        throw Error((diagnostics[stage]||diagnostics.DEVICE_CONNECTION_FAILED)
          +' (HTTP '+response.status+')');
      }
      const verified=await response.json();
      if(verified.enrolled_for_read!==true||verified.production_adopted!==false
        || verified.physical_writes_enabled!==false
        || verified.adoption_state!=='READ_ONLY_CONNECTED_LAB'
        || !Number.isSafeInteger(verified.card_count)||verified.card_count<1)
        throw Error('Physical device verification failed.');
      showStage('Saved to Device List','Connected · Read-only');
      diagnostic.textContent='AUTHENTICATED_DEVICE_READ_OK: Physical card inventory verified successfully.';
      output.textContent='CONNECTED (READ-ONLY) · '+verified.card_count
        +' active cards verified at '+verified.verified_at_utc+'.';
      await refresh();
      window.dispatchEvent(new Event('ipat-device-connection-changed'));
    }catch(error){
      showStage(draftSaved?'Saved to Device List':'Not Saved',draftSaved?'Pending · Connection attempt failed':'Save failed');
      diagnostic.textContent=error.message;
      output.textContent=(draftSaved?'DEVICE SAVED · Connection pending. ':'NOT SAVED · ')
        +error.message;
    }finally{
      busy=false;
      await refresh();
      validateProfile();
    }
  });
  // A saved draft waits for the missing lab-owner verification. Once the
  // operator supplies it, resume the exact same fixed-device connection.
  const resumeAfterVerification=()=>{
    if(!waitingForOwner || busy || !password.value || owner.value.trim().length<32)return;
    waitingForOwner=false;
    form.requestSubmit();
  };
  owner.addEventListener('change',resumeAfterVerification);
  owner.addEventListener('paste',()=>setTimeout(resumeAfterVerification,0));
  redrawModels();
  refresh();
  setInterval(refresh,15000);
})();
