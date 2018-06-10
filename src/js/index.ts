document.body.innerHTML += "HELLO";

interface App {
    name: string;
    volume: number;
}

var evtSource = new EventSource("/events");

evtSource.addEventListener("vol_update", function(e) {
        decode_payload(e).forEach(function(element){
                console.log(element);
                update_vol(element);
            });
    }, false);

function decode_payload(stin: string): App[]
{
    return JSON.parse(stin.data) as App[];
}

function update_vol(info: App)
{
    let volDiv  = document.getElementById('volume-container');
    let volBoxes = volDiv.children;
    for(let i = 0; i < volBoxes.length; i++)
    {
        let box = volBoxes[i];
        if(box.id == info.name)
        {
            set_volume(box, info.volume);
            return;
        }
    }
    console.log(info.name + " not found. Creating...");
    create_div(info);
}

function set_volume(volBox, volume: number)
{
    $("#slider" + volBox.id).slider("setValue", volume);
    //console.log("ID:" + volBox.id)
    //console.log("#slider" + volBox.id);
    //console.log($("#slider" + volBox.id).slider);
}

function create_div(info: App)
{
    let volDiv  = document.getElementById('volume-container');
    let sliderElement = make_slider(info);
    let sliderDiv = document.createElement("div");
    sliderDiv.id = info.name;
    sliderDiv.appendChild(sliderElement);
    volDiv.appendChild(sliderDiv);
    
    $("#" + sliderElement.id).slider({
        reversed : true
    });

    $("#" + sliderElement.id).slider('setValue', String(info.volume));

    $("#" + sliderElement.id).slider().on("change", slider_slid);
}

function slider_slid(ev)
{
    console.log(ev.target.id + " " + ev.target.value);
    $.post("setVol", {name: ev.target.id, number: ev.target.value} );
}

function make_slider(info: App)
{
    let fakeSlider = document.createElement("div");
//    fakeSlider.innerHTML = '<input type="range" min="1" max="100" value="50">';
    fakeSlider.innerHTML = '<input id="slider" type="text" data-slider-min="0" data-slider-max="100" data-slider-step="1" data-slider-value="-3" data-slider-orientation="vertical"/>';
    let slider = fakeSlider.firstChild;
    // TODO deal with [ and ] somehow 
    slider.id = "slider" + info.name;
    console.log("slider id: " + slider.id);

    return slider;
}
