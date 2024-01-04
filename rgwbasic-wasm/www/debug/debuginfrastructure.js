//"use strict";

import * as wasm from "rgwbasic-wasm";



const highlightLine = (root, lineNumber) => {
    let current = root.querySelector('.current-line');
    if (current) {
        current.classList.remove('current-line');
    }
    let elements = root.querySelectorAll(".codeLine");
    
    if (elements.length > lineNumber) {
        elements[lineNumber].classList.add('current-line');
    }
}

//window.addEventListener('load', () => {
setTimeout(() => {
    window.log = (str) => console.log(str);
    console.log("ready!!!");
    let iterpreter = null;
    let startbutton = document.getElementById('startButton');
    let stepButton = document.getElementById('stepButton');
    let lineInformatio = {};

    const turnCodeReadOnly = () => {
        let codeTextArea = document.getElementById('editablecodearea');
        codeTextArea.style.display = 'none';
        let readonlyDiv = document.getElementById('readonlyCode');
        readonlyDiv.style.display = 'block';
        

        let lines = codeTextArea.value.split('\n');
        readonlyDiv.replaceChildren.apply(
            readonlyDiv,
            lines.map((line) => {
                console.log(line);
                const lineDiv = document.createElement('DIV');
                lineDiv.classList.add('codeLine'); 
                lineDiv.appendChild(
                    document.createTextNode(line));
                return lineDiv;
            }));
    };

    startbutton.addEventListener('click', () => {
        turnCodeReadOnly();
        window.interpreter = wasm.GwInterpreterWrapper.new();
        let code = document.getElementById("editablecodearea").value;
        window.interpreter.load_from_string(code);
        console.log('Real vs source:');
        window.lineInformation = {};
        
        window.interpreter.real_vs_source_lines((r,s) => {            
            window.lineInformation[r] = s;
        });
        console.log(window.lineInformation);

        window.interpreter.start_step().then((resolved) =>{
            highlightLine(document.querySelector('#readonlyCode'),resolved);
        });
        
        //
    });
    
    stepButton.addEventListener('click', () => {        
        window.interpreter.step().then((resolved) =>{
            highlightLine(document.querySelector('#readonlyCode'),resolved);
        });
    });

    window.appendElementLd = function(uline) {
        //        window.consoleState.print(uline);
        console.log(`appendElementLd not implemented`);
    }

    window.appendElementLn = function(uline) {
        //        window.consoleState.println(uline);
        console.log(`appendElementLn not implemented ${uline}`);
    }

    window.log = function(str) {
        console.log(str);
    }

    window.readLine = function(continueFunc) {
  //      let txt = prompt();
        //        setTimeout(() => continueFunc(txt));
        console.log("Readline not implemented");
    }

});
