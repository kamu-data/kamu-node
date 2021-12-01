import { Component }        from '@angular/core';
import { DynamicComponent } from './dynamic.component';



@Component({
    selector:   'modal-image',

    template:   `
        <div class="modal__content" data-test-id="modal_image_content">
            <div class="modal__img text-center">
                <img data-test-id="modal_image" [src]="context.message">
            </div>

            <div class="text-center">
                <button class="modal__btn" data-test-id="yesButton"
                        (click)="context._close()">Close</button>
            </div>
        </div>
    `
})
export class ModalImageComponent extends DynamicComponent {}
