import {ModuleWithProviders, NgModule} from "@angular/core";
import {MatMenuModule} from "@angular/material/menu";
import {MatIconModule} from "@angular/material/icon";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";
import {SearchComponent} from "./search.component";
import {SearchAdditionalButtonsModule} from "../components/search-additional-buttons/search-additional-buttons.module";
import {MatToolbarModule} from "@angular/material/toolbar";
import {MatFormFieldModule} from "@angular/material/form-field";
import {MatTableModule} from "@angular/material/table";
import {MatSidenavModule} from "@angular/material/sidenav";
import {NgbModule} from "@ng-bootstrap/ng-bootstrap";

@NgModule({
    imports: [
        MatMenuModule,
        MatIconModule,
        MatButtonModule,
        MatToolbarModule,
        MatFormFieldModule,
        MatTableModule,
        MatSidenavModule,
        NgbModule,
        CommonModule,
        SearchAdditionalButtonsModule,
    ],
  exports: [SearchComponent],
  declarations: [SearchComponent]
})
export class SearchModule {
    public static forRoot(): ModuleWithProviders<any> {
    return { ngModule: SearchModule };
  }
}