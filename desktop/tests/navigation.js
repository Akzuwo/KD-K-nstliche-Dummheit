(async()=>{
 const report={passed:false,width:innerWidth,checks:[]};
 const sleep=ms=>new Promise(resolve=>setTimeout(resolve,ms));
 const wait=async(predicate,label)=>{const start=Date.now();while(Date.now()-start<12000){if(predicate())return;await sleep(50)}throw Error('Timeout: '+label)};
 const click=selector=>{const el=document.querySelector(selector);if(!el)throw Error('Fehlt: '+selector);el.click()};
 const input=(selector,text)=>{const el=document.querySelector(selector);el.value=text;el.dispatchEvent(new Event('input',{bubbles:true}))};
 try{
  await wait(()=>document.querySelector('.sidebar'),'Dioxus mounted');
  const sidebar=document.querySelector('.sidebar');
  report.initial_inert_attribute=sidebar.getAttribute('inert');report.initial_inert=sidebar.inert;
  if(sidebar.inert)throw Error('Geöffnete Sidebar ist gesperrt: inert='+sidebar.getAttribute('inert'));
  const nav=document.querySelector('.nav-actions .nav-button');nav.focus();if(document.activeElement!==nav)throw Error('Sidebar-Taste nimmt keinen Fokus an');
  const rect=nav.getBoundingClientRect();if(!nav.contains(document.elementFromPoint(rect.x+20,rect.y+rect.height/2)))throw Error('Sidebar-Taste nimmt keinen Zeigertreffer an');
  report.checks.push('open sidebar is focusable and hit-testable');
  click('.nav-actions .nav-button:nth-of-type(2)');await wait(()=>document.querySelector('#panel-title')?.textContent==='Modelle','model manager');click('.modal-header .icon-button');await wait(()=>!document.querySelector('.modal'),'close models');
  report.checks.push('Modelle opens and closes through Rust handler');
  click('.chat-title');await wait(()=>document.querySelector('.chat-row.selected'),'select conversation');
  click('.nav-actions .nav-button');await wait(()=>!document.querySelector('.chat-row.selected'),'new conversation');
  click('.chat-title');await wait(()=>document.querySelector('.chat-row.selected'),'select again');click('.brand-mark');await wait(()=>!document.querySelector('.chat-row.selected'),'brand new conversation');
  input('.search-wrap input','Abakus');await wait(()=>document.querySelectorAll('.chat-row').length===1,'full-text search');
  click('.chat-more');await wait(()=>document.querySelector('#rename'),'rename dialog');input('#rename','Umbenannt im echten WebView');await sleep(100);click('.dialog-actions .primary-button');await wait(()=>!document.querySelector('.modal'),'rename saves');
  input('.search-wrap input','Umbenannt');await wait(()=>document.querySelector('.chat-title')?.textContent.includes('Umbenannt'),'renamed list');
  report.checks.push('chat selection, new chat, KD button, search and rename');
  for(let i=0;i<3;i++){
   click('.sidebar-top .icon-button');await wait(()=>document.querySelector('.app.sidebar-closed'),'collapse');if(!sidebar.inert)throw Error('Geschlossene Sidebar nicht inert');nav.focus();if(document.activeElement===nav)throw Error('Versteckte Navigation im Fokus');
   click('.header-left .icon-button');await wait(()=>document.querySelector('.app.sidebar-open')&&!sidebar.inert,'reopen');nav.focus();if(document.activeElement!==nav)throw Error('Wieder geöffnete Sidebar bleibt gesperrt');
  }
  report.checks.push('collapse/reopen three times, no invisible focus targets');
  click('.profile-button');await wait(()=>document.querySelector('#panel-title')?.textContent==='Einstellungen','settings');
  await wait(()=>document.querySelector('#theme-switch'),'theme switch');
  let toggle=document.querySelector('#theme-switch');if(toggle.getAttribute('aria-checked')!=='true'){toggle.click();await wait(()=>document.documentElement.dataset.theme==='dark','dark start')}
  click('#theme-switch');await wait(()=>document.documentElement.dataset.theme==='light','switch light');await sleep(250);report.light=getComputedStyle(document.querySelector('main')).backgroundColor;
  click('#theme-switch');await wait(()=>document.documentElement.dataset.theme==='dark','switch dark');await sleep(250);report.dark=getComputedStyle(document.querySelector('main')).backgroundColor;
  if(report.light===report.dark)throw Error('Theme verändert keine Flächen');
  click('.modal-header .icon-button');await wait(()=>!document.querySelector('.modal'),'settings close');
  if(innerWidth<720){click('.sidebar-top .icon-button');await wait(()=>document.querySelector('.app.sidebar-closed'),'close mobile drawer');await sleep(350)}
  const headerToggle=document.querySelector('.header-theme-switch');const toggleRect=headerToggle.getBoundingClientRect();
  if(!headerToggle.contains(document.elementFromPoint(toggleRect.x+toggleRect.width/2,toggleRect.y+toggleRect.height/2)))throw Error('Theme-Schalter ist verdeckt');
  click('.header-theme-switch');await wait(()=>document.documentElement.dataset.theme==='light','header toggle light');click('.header-theme-switch');await wait(()=>document.documentElement.dataset.theme==='dark','header toggle dark');
  report.checks.push('profile settings, both theme switches, real light/dark render');
  if(document.querySelector('.error-toast'))throw Error(document.querySelector('.error-toast').textContent);
  report.passed=true;
 }catch(error){report.error=String(error);report.visible_text=document.body.innerText.slice(-1600)}
 dioxus.send(report);
})();
