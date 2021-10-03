import {ModuleWithProviders, NgModule} from "@angular/core";
import {CommonModule} from "@angular/common";
import {SearchSidenavComponent} from "./search-sidenav.component";
import {NgbModule, NgbNavModule} from "@ng-bootstrap/ng-bootstrap";
import {MatSidenavModule} from "@angular/material/sidenav";
import {MatFormFieldModule} from "@angular/material/form-field";
import {MatToolbarModule} from "@angular/material/toolbar";
import {MatButtonModule} from "@angular/material/button";
import {MatIconModule} from "@angular/material/icon";
import {MatMenuModule} from "@angular/material/menu";
import {FormsModule} from "@angular/forms";

@NgModule({
    imports: [
        CommonModule,
        MatMenuModule,
        MatIconModule,
        MatButtonModule,
        MatToolbarModule,
        MatFormFieldModule,
        MatSidenavModule,
        NgbModule,
        FormsModule,
        NgbNavModule
    ],
    exports: [SearchSidenavComponent],
    declarations: [SearchSidenavComponent]
})
export class SearchSidenavModule {
    public static forRoot(): ModuleWithProviders<any> {
        return {ngModule: SearchSidenavModule};
    }
}