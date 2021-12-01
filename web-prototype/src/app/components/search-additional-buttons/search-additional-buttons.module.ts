import { NgModule } from '@angular/core';
import {SearchAdditionalButtonsComponent} from "./search-additional-buttons.component";
import {MatMenuModule} from "@angular/material/menu";
import {MatIconModule} from "@angular/material/icon";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";
import {NgbPopoverModule} from "@ng-bootstrap/ng-bootstrap";


@NgModule({
    imports: [
        MatMenuModule,
        MatIconModule,
        MatButtonModule,
        CommonModule,
        NgbPopoverModule
    ],
  exports: [SearchAdditionalButtonsComponent],
  declarations: [SearchAdditionalButtonsComponent]
})
export class SearchAdditionalButtonsModule { }
