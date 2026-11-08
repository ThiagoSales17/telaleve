import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
const source = await readFile(new URL('../web/fullscreen.js', import.meta.url), 'utf8');
const { setupFullscreen } = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
function fixture() {
  const doc = new EventTarget(); const classes = new Set();
  doc.body = { classList: { toggle(name, active) { if (active) classes.add(name); else classes.delete(name); } } };
  const win = new EventTarget();const stack=[null];let position=0;
  const move=()=>{const event=new Event('popstate');Object.defineProperty(event,'state',{value:stack[position]});win.dispatchEvent(event);};
  win.history={pushState(state){stack.splice(++position);stack.push(state);},back(){if(position>0){position--;move();}},forward(){if(position+1<stack.length){position++;move();}}};
  doc.defaultView=win;
  const stage = { ownerDocument: doc };
  const button = () => ({ hidden: false, attrs: {}, setAttribute(k,v) { this.attrs[k]=v; } });
  const enter=button(), exit=button();
  doc.exitFullscreen=async()=>{doc.fullscreenElement=null;doc.dispatchEvent(new Event('fullscreenchange'));};
  return {doc,stage,enter,exit,classes,win,stack};
}
test('fullscreen targets only video stage and follows browser exit', async()=>{
 const f=fixture();let requested=false;
 f.stage.requestFullscreen=function(options){assert.equal(this,f.stage);assert.deepEqual(options,{navigationUI:'hide'});requested=true;f.doc.fullscreenElement=f.stage;f.doc.dispatchEvent(new Event('fullscreenchange'));return Promise.resolve();};
 setupFullscreen(f.stage,f.enter,f.exit);
 const pending=f.enter.onclick();assert.equal(requested,true,'request must occur in the tap handler before awaiting');await pending;
 assert.equal(f.enter.attrs['aria-pressed'],'true');
 await f.doc.exitFullscreen();assert.equal(f.enter.attrs['aria-pressed'],'false');
});
