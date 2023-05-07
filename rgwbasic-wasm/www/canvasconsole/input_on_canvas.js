import * as wasm from "rgwbasic-wasm";

class ConsoleManager {
   constructor(canvasElement, container) {
      this.initializeCanvas(canvasElement, container);
      this.maxLine = 25;
      this.currentLine = 0;
      this.top = 0;
      this.left = 0;
   }
   initializeCanvas(canvasElement, container) {
      let ctx = canvasElement.getContext('2d');
      ctx.font = "16px Monospace";

      let textMetrics = ctx.measureText('0');
      this.lineHeight = textMetrics.actualBoundingBoxAscent + textMetrics.actualBoundingBoxDescent;
      this.charWidth = textMetrics.actualBoundingBoxLeft + textMetrics.actualBoundingBoxRight;
      this.ctx = ctx;
      this.backgroundColor = 'white';


      // Add a little bit of extra space
      this.lineHeight += 4;

      let input = document.createElement('INPUT');
      input.style.font = ctx.font;
      container.appendChild(input);
      input.style.position = 'absolute';
      input.style.top = '0px';
      input.style.left = '0px';
      input.style.outline = 0;
      input.style.borderStyle = 'none';
      input.style.margin = 0;
      input.style.padding = 0;
      input.style.backgroundColor = 'rgba(0, 0, 0, 0)'
      input.spellcheck = false;
      input.focus();
      input.addEventListener('keyup', this.getEnterHandler(input,ctx));
      input.disabled = true;
      this.input = input;

   }
   print(text) {
      console.log(`__${this.left}, ${this.top} ${text}`);
      this.ctx.fillText(text, this.left, this.top + this.lineHeight);
      this.left += this.charWidth * text.length;
   }
   printline(text) {
      let ctx = this.ctx;
      let elementHeight = this.lineHeight;
      if ((this.currentLine + 1)  < this.maxLine) {
         this.top +=  elementHeight;
         this.input.style.top = `${this.top}px`;
         this.input.style.left = '0px';
         ctx.fillText(text,0,this.top);
      } else {
         ctx.fillText(text,0,this.top + elementHeight);
      }
      //element.value = '';
      this.incrementLine();
      this.left = 0;
   }

   getEnterHandler(element, ctx) {
      let elementHeight = this.lineHeight;
      return (ev) => {
         if (ev.key == 'Enter') {
            if (this.enterCallback) {
               this.enterCallback(element.value);
            }

             this.printline(element.value);
             element.value = '';
         }
      };
   }


   clear() {
      let color = this.ctx.fillStyle;
      this.ctx.fillStyle = this.backgroundColor;
      this.ctx.fillRect(0, 0, 300,300);
      this.ctx.fillStyle = color;
      this.currentLine = 0;
      this.top = 0;
      this.left = 0;
      this.input.style.top = `${this.top}px`;
   }

   readTextLine() {
      this.input.disabled = false;
      this.input.focus();
      return new Promise((resolve) => {
          this.enterCallback = (value) => {
              resolve(value);
             this.enterCallback = null;
             this.input.disabled = true;
          }
      });
   }

   scrollOneRow() {
      const beforeLastRow = (this.maxLine - 1) * this.lineHeight
     
      let img = this.ctx.getImageData(0,
                                      this.lineHeight, 
                                      300,
                                      beforeLastRow + this.lineHeight);
      this.ctx.putImageData(img, 0,0);
      let color = this.ctx.fillStyle;
      this.ctx.fillStyle = this.backgroundColor;
      this.ctx.fillRect(0, beforeLastRow + this.lineHeight, 300, this.lineHeight);
      this.ctx.fillStyle = color;
   }

   incrementLine() {
      if (this.currentLine + 1 >= this.maxLine) {
         this.scrollOneRow();
      } else {
         this.currentLine++;
      }
   }
}

setTimeout( () => {
   let manager = new ConsoleManager(
      document.getElementById('theCanvas'),
      document.getElementById('container'));
   window.appendElementLd = function(uline) {
      manager.print(uline);
   }

   window.appendElementLn = function(uline) {
      manager.printline(uline);
   }

   window.log = function(str) {
      console.log(str);
   }

   window.readLine = function(continueFunc) {
      manager.readTextLine().then(continueFunc);
   }

   let interpreter = wasm.GwInterpreterWrapper.new(); 

   window.interpreter = interpreter;
    const replLoop = () => {
        manager.printline("Ok");
        manager.readTextLine().then((line) => {
         if (line.toLowerCase() == 'run') {
             interpreter.run_evaluator_loop(interpreter);
         } else {
             interpreter.eval_in_interpreter(line);
             requestAnimationFrame(replLoop);
         }
      });
   };
   requestAnimationFrame(replLoop);
});
