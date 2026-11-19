(()=>{
 if(window.kdInstalled)return;window.kdInstalled=true;
 window.kdCopy=async text=>{try{await navigator.clipboard.writeText(text)}catch{const t=document.createElement('textarea');t.value=text;t.style.position='fixed';t.style.opacity='0';document.body.append(t);t.select();document.execCommand('copy');t.remove()}};
 let follow=true;
 document.addEventListener('scroll',e=>{if(e.target.id==='conversation'){const t=e.target;follow=t.scrollHeight-t.scrollTop-t.clientHeight<140}},true);
 document.addEventListener('keydown',e=>{if(e.ctrlKey&&e.key.toLowerCase()==='n'){e.preventDefault();document.querySelector('.nav-button')?.click()}if(e.key==='Escape')document.querySelector('.modal-header .icon-button')?.click()});
 document.addEventListener('click',e=>{const a=e.target.closest('.markdown a');if(a){e.preventDefault();window.kdCopy(a.href);a.title='Link kopiert. Zum Öffnen in den Browser einfügen.'}});
 const observer=new MutationObserver(()=>{
  const textarea=document.querySelector('#prompt');if(textarea){textarea.style.height='auto';textarea.style.height=Math.min(textarea.scrollHeight,200)+'px'}
  document.querySelectorAll('.markdown pre').forEach(pre=>{if(pre.querySelector('button'))return;const code=pre.querySelector('code');const bar=document.createElement('div');bar.className='code-header';const lang=document.createElement('span');lang.textContent=code?.className.replace('language-','')||'Code';const button=document.createElement('button');button.className='code-copy';button.textContent='Kopieren';button.onclick=()=>{window.kdCopy(code.textContent);button.textContent='Kopiert';setTimeout(()=>button.textContent='Kopieren',1400)};bar.append(lang,button);pre.prepend(bar)});
  const scroller=document.querySelector('#conversation');if(follow&&scroller)scroller.scrollTop=scroller.scrollHeight;
  const modal=document.querySelector('.modal');if(modal&&!modal.dataset.focused){modal.dataset.focused='1';modal.querySelector('button,input,select')?.focus()}
 });
 observer.observe(document.body,{childList:true,subtree:true,characterData:true});
 document.addEventListener('keydown',e=>{const m=document.querySelector('.modal');if(e.key!=='Tab'||!m)return;const f=[...m.querySelectorAll('button:not(:disabled),input:not(:disabled),select:not(:disabled)')];if(e.shiftKey&&document.activeElement===f[0]){e.preventDefault();f.at(-1).focus()}else if(!e.shiftKey&&document.activeElement===f.at(-1)){e.preventDefault();f[0].focus()}});
})();
