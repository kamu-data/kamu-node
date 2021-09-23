import { ModuleWithProviders, NgModule } from '@angular/core';
import {SearchAdditionalButtonsComponent} from "./search-additional-buttons.component";
import {SearchAdditionalButtonsService} from "./search-additional-buttons.service";
import {MatMenuModule} from "@angular/material/menu";
import {MatIconModule} from "@angular/material/icon";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";


@NgModule({
    imports: [
        MatMenuModule,
        MatIconModule,
        MatButtonModule,
        CommonModule
    ],
  exports: [SearchAdditionalButtonsComponent],
  declarations: [SearchAdditionalButtonsComponent]
})
export class SearchAdditionalButtonsModule { }
