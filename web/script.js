'use strict';

let is_windows = null;
let channels = null;
function addSlider(channel_name) {
    channels.insertAdjacentElement(
        'beforeend',
        createSlider(channel_name)
    );
    let slider = document.getElementById(channel_name+"_slider");
    slider.addEventListener(is_windows?'change':'input',function(){
            external.invoke("change_volume:"+channel_name+":"+this.value);
        },
        false
    );
}
function createSlider(channel_name) {
    let slider = document.createElement("tr");
    slider.className="w3-row";
    slider.insertAdjacentHTML(
        'afterbegin',
        "<td class='w3-center' style='width:50px'><h4>"+channel_name+"</h4></td>"+
        "<td class='w3-rest'>"+
            "<input type='range'"+
                "id='"+channel_name+"_slider'"+
                "min='0'"+
                "max='100'"+
                "value='100'"+
            "/>"+
        "</td>"
    );
    return slider;
}
function setSliderValue(channel_name, value) {
    let slider = document.getElementById(channel_name+"_slider");
    if (slider != null) slider.value = value;
}
function clearSliders() {
    while (channels.firstChild)
        channels.removeChild(channels.firstChild);
}

let alerts_footer = null;
let alerts = null;
function addAlert(name, color, text) {
    removeAlert(name);
    let new_alert = createAlert(name, color, text);
    alerts[name] = new_alert;
    alerts_footer.insertAdjacentElement('afterbegin', new_alert);
    if (alerts_footer.childElementCount > 10)
        removeAlert(alerts_footer.lastChild.name);
}
function removeAlert(name) {
    let alert = document.getElementById("alert_"+name);
    if (alert != null) {
        alerts_footer.removeChild(alert);
        alerts.delete(name);
    }
}
function createAlert(name, color, text) {
    let alert=document.createElement("div");
    alert.name = name;
    alert.id="alert_"+name;
    alert.className="w3-container w3-animate-bottom w3-"+color;
    alert.innerHTML=text;
    alert.timer = 2.5;

    let cross = document.createElement("span");
    cross.className="w3-closebtn";
    cross.setAttribute("onclick", "removeAlert('"+name+"')");
    cross.innerHTML="&times;";

    alert.insertAdjacentElement('beforeend',cross);
    
    return alert;
}

function main() {
    channels = document.getElementById('channels');
    is_windows = /MSIE|Trident|Edge/.test(window.navigator.userAgent);
    alerts_footer = document.getElementById('alerts');
    alerts = new Map();
    
    let prev = null;
    function step(now) {
        let dt = (prev!=null) ? (now-prev)*0.001 : 0.0;
        prev = now;
        for (let key in alerts) {
            let alert = alerts[key];
            alert.timer -= dt;
            if (alert.timer <= 1.0) alert.style.opacity = alert.timer;
            if (alert.timer <= 0.0) removeAlert(alert.name);
        }
        window.requestAnimationFrame(step);
    }
    window.requestAnimationFrame(step);
}