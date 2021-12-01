import {Component} from '@angular/core';
import { DynamicComponent } from './dynamic.component';



@Component({
    selector:   'modal-dialog',
    template:   `
        <div class="modal__content" (click)="hideAll()">
            <div [ngClass]="context.bigTextBlock ? 'modal__bigtest-dialog' : 'modal__dialog'">

                <h2 class="modal__header" data-test-id="modalHeader"
                    [ngClass]="{
                        'modal__header-ok':         context.status === 'ok',
                        'modal__header-warning':    context.status === 'warning',
                        'modal__header-error':      context.status === 'error',
                        'modal__header-black':      context.status === 'dialog_question'
                    }">{{context.title}}</h2>

                <p *ngIf="context.message" class="modal__msg" data-test-id="modalMessage">{{context.message}}</p>

                <button [class]="context.lastButtonText || context.tooLastButtonText ? 'modal__btn modal__btn-first' :'modal__btn'"
                        data-test-id="yesButton" *ngIf="context.yesButtonText"
                        (click)="onClick(true, context.locationBack)">{{context.yesButtonText}}</button>

                <button [class]="context.lastButtonText || context.tooLastButtonText ? 'modal__btn modal__btn-last' :'modal__btn'"
                        data-test-id="noButton"
                        *ngIf="context.noButtonText"
                        (click)="onClick(false)">{{context.noButtonText}}</button>
                <button class="modal__btn modal__btn-last" data-test-id="lastButton"
                        *ngIf="context.lastButtonText"
                        (click)="onClick('last')">{{context.lastButtonText}}</button>
                <button class="modal__btn modal__btn-last" data-test-id="tooLastButtonText"
                        *ngIf="context.tooLastButtonText"
                        (click)="onClick('tooLast')">{{context.tooLastButtonText}}</button>

            </div>
        </div>
    `
})
export class ModalDialogComponent extends DynamicComponent {

    onClick(action: boolean | string, locationBack?: boolean) {
        this.context._close(locationBack);
        this.context.handler(action);
    }
    hideAll() {
        // tslint:disable-next-line:no-unused-expression
        this.context.title === 'Search for:' && this.context._close();
    }
}
