import {ModuleWithProviders, NgModule} from "@angular/core";
import {MatIconModule} from "@angular/material/icon";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";
import {NgbModule} from "@ng-bootstrap/ng-bootstrap";
import {FormsModule} from "@angular/forms";
import {MatChipsModule} from "@angular/material/chips";
import {MatDividerModule} from '@angular/material/divider';
import {PaginationComponent} from "./pagination.component";

@NgModule({
    imports: [
        MatIconModule,
        MatButtonModule,
        MatDividerModule,
        NgbModule,
        CommonModule,
        FormsModule,
        MatChipsModule,
    ],
    exports: [PaginationComponent],
    declarations: [PaginationComponent]
})
export class PaginationModule {
    // tslint:disable-next-line: no-any
    public static forRoot(): ModuleWithProviders<any> {
        return {ngModule: PaginationModule};
    }
}